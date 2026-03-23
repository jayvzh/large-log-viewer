use crate::database::Database;
use crate::database::tokenize;

use crate::highlight_engine::HighlightEngine;
use crate::highlight_store::HighlightStore;
use crate::models::*;
use crate::parser::LogParser;
use crate::parser::LogTemplateParser;
use crate::reader::LogFileReader;
use crate::template_store::TemplateStore;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

pub mod file_association;
pub use file_association::*;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024;

pub struct AppState {
    db: Arc<Database>,
    template_store: Arc<TemplateStore>,
    highlight_store: Arc<HighlightStore>,
    highlight_engine: RwLock<HighlightEngine>,
    current_file: RwLock<Option<FileInfo>>,
    next_file_id: RwLock<u64>,
    template_parser: RwLock<Option<Arc<LogTemplateParser>>>,
    cached_template_name: RwLock<Option<String>>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            template_store: Arc::new(TemplateStore::new()),
            highlight_store: Arc::new(HighlightStore::new()),
            highlight_engine: RwLock::new(HighlightEngine::new(vec![])),
            current_file: RwLock::new(None),
            next_file_id: RwLock::new(1),
            template_parser: RwLock::new(None),
            cached_template_name: RwLock::new(None),
        }
    }
    
    async fn get_or_create_parser(&self, template_name: Option<&str>, lines_for_detection: Option<&[&str]>) -> Result<Option<Arc<LogTemplateParser>>, String> {
        let _cached_name = self.cached_template_name.read().await.clone();
            
            // 总是创建新的解析器，确保模板选择生效
            let parser = if let Some(name) = template_name {
                let templates = self.template_store.get_all_templates().await?;
                let selected_template = templates.iter().find(|t| &t.name == name);
                if let Some(t) = selected_template {
                    // 只使用选中的模板，不添加其他模板
                    let prioritized_templates = vec![t.clone()];
                    Some(Arc::new(LogTemplateParser::new(prioritized_templates)?))
                } else {
                    return Err(format!("Template not found: {}", name));
                }
            } else if let Some(lines) = lines_for_detection {
                let templates = self.template_store.get_all_templates().await?;
                if !templates.is_empty() {
                    let detector = LogTemplateParser::new(templates)?;
                    let results = detector.detect_best_template(lines);
                    if let Some(best) = results.first() {
                        if best.match_rate > 0.5 {
                            let templates = self.template_store.get_all_templates().await?;
                            let template = templates.iter().find(|t| &t.name == &best.template_name);
                            if let Some(t) = template {
                                // 只使用检测到的模板，不添加其他模板
                                let prioritized_templates = vec![t.clone()];
                                let parser = Arc::new(LogTemplateParser::new(prioritized_templates)?);
                                {
                                    let mut cached_name = self.cached_template_name.write().await;
                                    *cached_name = Some(t.name.clone());
                                }
                                return Ok(Some(parser));
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
        
        {
            let mut cached_parser = self.template_parser.write().await;
            *cached_parser = parser.clone();
        }
        {
            let mut cached_name = self.cached_template_name.write().await;
            *cached_name = template_name.map(|s| s.to_string());
        }
        
        Ok(parser)
    }
    
    async fn clear_parser_cache(&self) {
        let mut parser = self.template_parser.write().await;
        *parser = None;
        let mut name = self.cached_template_name.write().await;
        *name = None;
    }

    async fn update_highlight_engine(&self) -> Result<(), String> {
        let profiles = self.highlight_store.get_all_profiles().await?;
        let mut engine = self.highlight_engine.write().await;
        engine.update_profiles(profiles);
        Ok(())
    }
}

#[tauri::command]
pub async fn open_file(
    path: String,
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<FileInfo, String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    let metadata = std::fs::metadata(&path_buf)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {:.2} GB. Maximum allowed size is 10 GB.",
            metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0)
        ));
    }
    
    let name = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let mut next_id = state.next_file_id.write().await;
    let file_id = *next_id;
    *next_id += 1;
    
    let file_info = FileInfo {
        id: file_id,
        path: path.clone(),
        name,
        size: metadata.len(),
        entry_count: 0,
        loaded_at: chrono::Utc::now().timestamp_millis(),
    };
    
    state.db.store_file_info(&file_info).await?;
    
    {
        let mut current_file = state.current_file.write().await;
        *current_file = Some(file_info.clone());
    }
    
    Ok(file_info)
}

#[tauri::command]
pub async fn parse_log(
    file_id: u64,
    template_name: Option<String>,
    encoding: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ParseResult, String> {
    state.db.clear_file_data(file_id).await?;
    
    let path = {
        let current_file = state.current_file.read().await;
        let file_info = current_file.as_ref()
            .ok_or("No file loaded")?;
        PathBuf::from(&file_info.path)
    };
    
    let settings = get_settings().await.unwrap_or_default();
    if let Some(workers) = settings.parallel_workers {
        if workers > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build_global()
                .ok();
        }
    }
    
    let encoding_str = encoding.unwrap_or_else(|| settings.encoding);
    let file_encoding = crate::reader::FileEncoding::from_str(&encoding_str);
    
    let reader = LogFileReader::with_encoding(path, file_encoding)?;
    let file_size = reader.file_size();
    
    let mut reader = reader.read_lines_mmap()?;
    
    let _ = app.emit("parse_progress", ParseProgress {
        file_id,
        total_bytes: file_size,
        processed_bytes: 0,
        entries_parsed: 0,
        percentage: 0.0,
        phase: "收集行数据中...".to_string(),
        is_complete: false,
    });
    
    let mut all_lines: Vec<(Vec<u8>, u64, u64)> = Vec::new();
    while let Some(line) = reader.next_line() {
        all_lines.push((line.data.to_vec(), line.line_number, line.offset));
    }
    let total_lines = all_lines.len();
    
    let detection_lines: Vec<String> = all_lines
        .iter()
        .take(100)
        .map(|(data, _, _)| LogFileReader::decode_line(data, file_encoding))
        .collect();
    let detection_refs: Vec<&str> = detection_lines.iter().map(|s| s.as_str()).collect();
    
    let template_parser = state.get_or_create_parser(template_name.as_deref(), Some(&detection_refs)).await?;
    let parser = LogParser::new(file_id);
    
    let mut level_bitmaps: [RoaringBitmap; 7] = Default::default();
    for i in 0..7 {
        level_bitmaps[i] = RoaringBitmap::new();
    }
    
    let mut word_index: HashMap<u64, RoaringBitmap> = HashMap::new();
    
    let entry_count = AtomicU64::new(0);
    let mut last_update = Instant::now();
    const UPDATE_INTERVAL_MS: u64 = 100;
    const BATCH_SIZE: usize = 10000;
    
    for chunk in all_lines.chunks(BATCH_SIZE) {
        let chunk_lines: Vec<(String, u64, u64)> = chunk
            .iter()
            .map(|(data, line_number, offset)| {
                let line_str = LogFileReader::decode_line(data, file_encoding);
                (line_str, *line_number, *offset)
            })
            .collect();
        
        let chunk_refs: Vec<(&str, u64, u64)> = chunk_lines
            .iter()
            .map(|(s, ln, off)| (s.as_str(), *ln, *off))
            .collect();
        
        // 根据是否使用模板解析器选择解析方式
        let parsed_entries: Vec<LogEntry> = if let Some(ref tp) = template_parser {
            chunk_refs.iter().map(|(line, line_number, offset)| {
                let event = tp.parse_line(line);
                
                let has_timestamp = event.timestamp.is_some();
                let has_level = event.level.is_some();
                let has_extra = !event.extra.is_empty();
                let is_parsed = has_timestamp || has_level || has_extra;
                
                let timestamp = event.timestamp
                    .and_then(|ts: String| {
                        let s = ts.replace(',', ".");
                        
                        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.3f") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S.%f") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%f") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S.%f%:z") {
                            Some(dt.and_utc().timestamp_millis())
                        } else if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%d/%b/%Y:%H:%M:%S %z") {
                            Some(dt.timestamp_millis())
                        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%d/%b/%Y:%H:%M:%S") {
                            Some(dt.and_utc().timestamp_millis())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                
                let level = event.level
                    .map(|l: String| LogLevel::from_str(&l))
                    .unwrap_or(LogLevel::Other);
                
                let source = event.source.unwrap_or_default();
                let summary = if event.message.is_empty() { line.to_string() } else { event.message.clone() };
                
                LogEntry::with_extra(
                    *line_number,
                    file_id,
                    *line_number,
                    timestamp,
                    level,
                    &source,
                    &summary,
                    *offset,
                    line.len() as u32,
                    event.extra,
                    is_parsed,
                )
            }).collect()
        } else {
            parser.parse_lines_parallel(&chunk_refs)
        };
        
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut local_bitmaps: [RoaringBitmap; 7] = Default::default();
        for i in 0..7 {
            local_bitmaps[i] = RoaringBitmap::new();
        }
        
        let mut local_word_index: HashMap<u64, RoaringBitmap> = HashMap::new();
        let mut time_index_batch: Vec<(i64, u64)> = Vec::new();
        let mut extra_field_indices: HashMap<String, HashMap<String, RoaringBitmap>> = HashMap::new();
        
        for mut entry in parsed_entries {
            let current_id = entry_count.fetch_add(1, Ordering::SeqCst);
            entry.id = current_id;
            
            let level_idx = entry.level as usize;
            if level_idx < 7 {
                local_bitmaps[level_idx].insert(current_id as u32);
            }
            
            let text = format!("{} {}", entry.source_str(), entry.message_str());
            let words = tokenize(&text);
            for word in words {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                word.hash(&mut hasher);
                let hash = hasher.finish();
                local_word_index
                    .entry(hash)
                    .or_default()
                    .insert(current_id as u32);
            }
            
            if entry.timestamp > 0 {
                time_index_batch.push((entry.timestamp, current_id));
            }
            
            for (field_name, field_value) in &entry.extra {
                extra_field_indices
                    .entry(field_name.clone())
                    .or_default()
                    .entry(field_value.clone())
                    .or_default()
                    .insert(current_id as u32);
            }
            
            batch.push(entry);
        }
        
        state.db.store_entries_batch(&batch).await?;
        
        if !time_index_batch.is_empty() {
            state.db.store_time_index_batch(file_id, &time_index_batch).await?;
        }
        
        for (field_name, value_index) in &extra_field_indices {
            state.db.store_extra_field_index_batch(file_id, field_name, value_index).await?;
        }
        
        for i in 0..7 {
            level_bitmaps[i] |= &local_bitmaps[i];
        }
        
        for (hash, bitmap) in local_word_index {
            let global_bitmap = word_index.entry(hash).or_default();
            *global_bitmap |= &bitmap;
        }
        
        let now = Instant::now();
        let elapsed = now.duration_since(last_update).as_millis() as u64;
        
        if elapsed >= UPDATE_INTERVAL_MS {
            last_update = now;
            let current_count = entry_count.load(Ordering::SeqCst);
            let progress = (current_count as f32 / total_lines as f32) * 100.0;
            
            let phase = format!("并行解析中: 已处理 {} 条", current_count);
            
            let _ = app.emit("parse_progress", ParseProgress {
                file_id,
                total_bytes: file_size,
                processed_bytes: (progress / 100.0 * file_size as f32) as u64,
                entries_parsed: current_count,
                percentage: progress,
                phase,
                is_complete: false,
            });
        }
    }
    
    let final_count = entry_count.load(Ordering::SeqCst);
    
    for (i, bitmap) in level_bitmaps.iter().enumerate() {
        let level = match i {
            0 => LogLevel::Fatal,
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            5 => LogLevel::Trace,
            _ => LogLevel::Other,
        };
        state.db.store_level_bitmap(file_id, level, bitmap).await?;
    }
    
    let _ = app.emit("parse_progress", ParseProgress {
        file_id,
        total_bytes: file_size,
        processed_bytes: file_size,
        entries_parsed: final_count,
        percentage: 95.0,
        phase: "构建搜索索引...".to_string(),
        is_complete: false,
    });
    
    state.db.build_search_index_from_map(file_id, &word_index).await?;
    
    let _ = app.emit("parse_progress", ParseProgress {
        file_id,
        total_bytes: file_size,
        processed_bytes: file_size,
        entries_parsed: final_count,
        percentage: 100.0,
        phase: "完成".to_string(),
        is_complete: true,
    });
    
    let detected_template = state.cached_template_name.read().await.clone();
    
    let mut current_file = state.current_file.write().await;
    if let Some(ref mut info) = *current_file {
        info.entry_count = final_count;
    }
    drop(current_file);
    
    Ok(ParseResult {
        entry_count: final_count,
        detected_template,
    })
}

#[tauri::command]
pub async fn get_entries(
    file_id: u64,
    offset: u64,
    limit: u64,
    highlight_profile: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let entries = state.db.get_entries(file_id, offset, limit).await?;
    let mut views: Vec<LogEntryView> = entries.iter().map(LogEntryView::from).collect();
    
    if let Some(profile_name) = highlight_profile {
        let engine = state.highlight_engine.read().await;
        for (view, entry) in views.iter_mut().zip(&entries) {
            view.highlight_spans = engine.process_entry(entry, &profile_name);
        }
    }
    
    Ok(views)
}

#[tauri::command]
pub async fn export_logs(
    file_id: u64,
    output_path: String,
    format: String,
    level: Option<String>,
    query: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let mut result_bitmap: Option<RoaringBitmap> = None;
    
    if let Some(l) = level {
        let level_enum = LogLevel::from_str(&l);
        let level_bitmap = state.db.get_level_bitmap(file_id, level_enum).await?
            .unwrap_or_default();
        result_bitmap = Some(level_bitmap);
    }
    
    if let (Some(start), Some(end)) = (start_time, end_time) {
        let time_bitmap = state.db.filter_by_time_indexed(file_id, start, end).await?;
        result_bitmap = match result_bitmap {
            Some(b) => Some(b & time_bitmap),
            None => Some(time_bitmap),
        };
    }
    
    if let Some(q) = query {
        let search_bitmap = state.db.search_bitmap(file_id, &q).await?;
        result_bitmap = match result_bitmap {
            Some(b) => Some(b & search_bitmap),
            None => Some(search_bitmap),
        };
    }
    
    let ids: Vec<u64> = match result_bitmap {
        Some(bitmap) => bitmap.iter().map(|id| id as u64).collect(),
        None => {
            let entries = state.db.get_all_entries(file_id).await?;
            entries.iter().map(|e| e.id).collect()
        }
    };
    
    let entries = state.db.get_entries_by_ids(file_id, &ids).await?;
    
    let content = match format.as_str() {
        "json" => {
            let views: Vec<LogEntryView> = entries.iter().map(LogEntryView::from).collect();
            serde_json::to_string_pretty(&views)
                .map_err(|e| format!("Failed to serialize: {}", e))?
        }
        _ => {
            entries.iter()
                .map(|e| {
                    let ts = if e.timestamp > 0 {
                        chrono::DateTime::from_timestamp_millis(e.timestamp)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                            .unwrap_or_else(|| e.timestamp.to_string())
                    } else {
                        "-".to_string()
                    };
                    format!("{} | {} | {} | {}", ts, e.level.as_str(), e.source_str(), e.message_str())
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    };
    
    fs::write(&output_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(entries.len() as u64)
}

#[tauri::command]
pub async fn get_templates(state: State<'_, AppState>) -> Result<Vec<LogTemplate>, String> {
    state.template_store.get_all_templates().await
}

#[tauri::command]
pub async fn create_template(
    mut template: LogTemplate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if template.is_builtin {
        return Err("Cannot create builtin template".to_string());
    }
    let (extra_fields, has_level, has_timestamp, has_source) = 
        LogTemplateParser::extract_fields_from_pattern(&template.pattern);
    template.extra_fields = extra_fields;
    template.has_level = has_level;
    template.has_timestamp = has_timestamp;
    template.has_source = has_source;
    state.template_store.add_template(&template).await
}

#[tauri::command]
pub async fn update_template(
    mut template: LogTemplate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if template.is_builtin {
        return Err("Cannot update builtin template".to_string());
    }
    let (extra_fields, has_level, has_timestamp, has_source) = 
        LogTemplateParser::extract_fields_from_pattern(&template.pattern);
    template.extra_fields = extra_fields;
    template.has_level = has_level;
    template.has_timestamp = has_timestamp;
    template.has_source = has_source;
    state.clear_parser_cache().await;
    state.template_store.add_template(&template).await
}

#[tauri::command]
pub async fn delete_template(
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(t) = state.template_store.get_template(&name).await? {
        if t.is_builtin {
            return Err("Cannot delete builtin template".to_string());
        }
    }
    state.clear_parser_cache().await;
    state.template_store.remove_template(&name).await
}

#[tauri::command]
pub async fn test_template(
    pattern: String,
    test_line: String,
) -> Result<TemplateTestResult, String> {
    Ok(LogTemplateParser::test_pattern(&pattern, &test_line))
}

#[tauri::command]
pub async fn detect_template(
    file_path: String,
    encoding: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DetectResult>, String> {
    let templates = state.template_store.get_all_templates().await?;
    let parser = LogTemplateParser::new(templates)?;
    
    let settings = get_settings().await.unwrap_or_default();
    let encoding_str = encoding.unwrap_or_else(|| settings.encoding);
    let file_encoding = crate::reader::FileEncoding::from_str(&encoding_str);
    
    let reader = LogFileReader::with_encoding(PathBuf::from(&file_path), file_encoding)?;
    let mut lines: Vec<String> = Vec::new();
    let mut reader = reader.read_lines_mmap()?;
    
    while let Some(line) = reader.next_line() {
        if lines.len() >= 100 {
            break;
        }
        lines.push(LogFileReader::decode_line(&line.data, file_encoding));
    }
    
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    Ok(parser.detect_best_template(&refs))
}

#[tauri::command]
pub async fn export_templates(state: State<'_, AppState>) -> Result<String, String> {
    let templates = state.template_store.get_all_templates().await?;
    serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("Failed to export templates: {}", e))
}

#[tauri::command]
pub async fn import_templates(
    json: String,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let templates: Vec<LogTemplate> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse templates: {}", e))?;
    
    let mut count = 0u32;
    for mut t in templates {
        t.is_builtin = false;
        t.created_at = chrono::Utc::now().timestamp_millis();
        t.updated_at = t.created_at;
        state.template_store.add_template(&t).await?;
        count += 1;
    }
    
    Ok(count)
}

#[tauri::command]
pub async fn set_file_association(extension: String) -> Result<(), String> {
    windows::set_file_association(&extension)
}

#[tauri::command]
pub async fn set_file_associations(extensions: Vec<String>) -> Result<(), String> {
    windows::set_file_associations(&extensions)
}

#[tauri::command]
pub async fn remove_file_association(extension: String) -> Result<(), String> {
    windows::remove_file_association(&extension)
}

#[tauri::command]
pub async fn remove_file_associations(extensions: Vec<String>) -> Result<(), String> {
    windows::remove_file_associations(&extensions)
}

#[tauri::command]
pub async fn check_file_association() -> Result<FileAssociationStatus, String> {
    let log = windows::check_file_association("log")?;
    Ok(FileAssociationStatus { log })
}

#[tauri::command]
pub async fn check_file_associations() -> Result<std::collections::HashMap<String, bool>, String> {
    windows::check_file_associations()
}

#[tauri::command]
pub async fn get_entry_detail(
    file_id: u64,
    entry_id: u64,
    highlight_profile: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<LogEntryView>, String> {
    let entry = state.db.get_entry(file_id, entry_id).await?;
    
    match entry {
        Some(e) => {
            let file_path = {
                let current_file = state.current_file.read().await;
                current_file.as_ref()
                    .map(|f| PathBuf::from(&f.path))
            };
            
            let raw = if let Some(path) = file_path {
                let reader = LogFileReader::new(path)?;
                reader.read_raw_content(e.raw_offset, e.raw_length)?
            } else {
                String::new()
            };
            
            let mut view = LogEntryView::from(&e);
            view.raw = raw;
            
            if let Some(profile_name) = highlight_profile {
                let engine = state.highlight_engine.read().await;
                view.highlight_spans = engine.process_entry(&e, &profile_name);
            }
            
            Ok(Some(view))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_stats(
    file_id: u64,
    state: State<'_, AppState>,
) -> Result<LogStats, String> {
    state.db.get_stats(file_id).await
}

#[tauri::command]
pub async fn filter_by_level(
    file_id: u64,
    level: String,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let level = LogLevel::from_str(&level);
    let bitmap = state.db.get_level_bitmap(file_id, level).await?
        .unwrap_or_default();
    
    let mut entries = Vec::with_capacity(limit as usize);
    
    for id in bitmap.iter().skip(offset as usize).take(limit as usize) {
        if let Some(entry) = state.db.get_entry(file_id, id as u64).await? {
            entries.push(LogEntryView::from(&entry));
        }
    }
    
    Ok(entries)
}

#[tauri::command]
pub async fn search(
    file_id: u64,
    query: String,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let entries = state.db.search_with_index(file_id, &query, offset, limit).await?;
    
    Ok(entries.iter().map(LogEntryView::from).collect())
}

#[tauri::command]
pub async fn filter_by_time(
    file_id: u64,
    start_time: i64,
    end_time: i64,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let bitmap = state.db.filter_by_time_indexed(file_id, start_time, end_time).await?;
    
    let entries = state.db.get_entries_by_bitmap(file_id, &bitmap, offset, limit).await?;
    
    Ok(entries.iter().map(LogEntryView::from).collect())
}

#[tauri::command]
pub async fn get_current_file(state: State<'_, AppState>) -> Result<Option<FileInfo>, String> {
    let file = state.current_file.read().await;
    Ok(file.clone())
}

#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> Result<CacheInfo, String> {
    let count = state.db.clear_cache().await?;
    
    let mut current_file = state.current_file.write().await;
    *current_file = None;
    
    Ok(CacheInfo {
        entries_cleared: count,
        data_dir: Database::get_data_dir().to_string_lossy().to_string(),
        cache_size: 0,
    })
}

#[tauri::command]
pub async fn get_cache_info() -> Result<CacheInfo, String> {
    let data_dir = Database::get_data_dir();
    let db_path = data_dir.join("logs.db");
    
    let size = if db_path.exists() {
        std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };
    
    Ok(CacheInfo {
        entries_cleared: 0,
        data_dir: data_dir.to_string_lossy().to_string(),
        cache_size: size,
    })
}

fn get_settings_path() -> PathBuf {
    let data_dir = Database::get_data_dir();
    data_dir.join("settings.json")
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        let settings: AppSettings = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn filter_logs(
    file_id: u64,
    level: Option<String>,
    query: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    offset: u64,
    limit: u64,
    extra_conditions: Option<Vec<ExtraFilterCondition>>,
    extra_combine_mode: Option<String>,
    highlight_profile: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let mut result_bitmap: Option<RoaringBitmap> = None;
    
    if let Some(l) = level {
        let level_enum = LogLevel::from_str(&l);
        let level_bitmap = state.db.get_level_bitmap(file_id, level_enum).await?
            .unwrap_or_default();
        result_bitmap = Some(level_bitmap);
    }
    
    if let (Some(start), Some(end)) = (start_time, end_time) {
        let time_bitmap = state.db.filter_by_time_indexed(file_id, start, end).await?;
        result_bitmap = match result_bitmap {
            Some(b) => Some(b & time_bitmap),
            None => Some(time_bitmap),
        };
    }
    
    if let Some(q) = query {
        let search_bitmap = state.db.search_bitmap(file_id, &q).await?;
        result_bitmap = match result_bitmap {
            Some(b) => Some(b & search_bitmap),
            None => Some(search_bitmap),
        };
    }
    
    if let Some(conditions) = extra_conditions {
        if !conditions.is_empty() {
            let combine_mode = extra_combine_mode.unwrap_or_else(|| "and".to_string());
            let conditions_tuple: Vec<(String, String, String)> = conditions
                .iter()
                .map(|c| (c.field.clone(), c.operator.clone(), c.value.clone()))
                .collect();
            let extra_bitmap = state.db.filter_by_extra_conditions(file_id, &conditions_tuple, &combine_mode).await?;
            result_bitmap = match result_bitmap {
                Some(b) => Some(b & extra_bitmap),
                None => Some(extra_bitmap),
            };
        }
    }
    
    let entries = match result_bitmap {
        Some(bitmap) => {
            let ids: Vec<u64> = bitmap.iter()
                .skip(offset as usize)
                .take(limit as usize)
                .map(|id| id as u64)
                .collect();
            state.db.get_entries_by_ids(file_id, &ids).await?
        }
        None => state.db.get_entries(file_id, offset, limit).await?
    };
    
    let mut views: Vec<LogEntryView> = entries.iter().map(LogEntryView::from).collect();
    
    if let Some(profile_name) = highlight_profile {
        let engine = state.highlight_engine.read().await;
        for (view, entry) in views.iter_mut().zip(&entries) {
            view.highlight_spans = engine.process_entry(entry, &profile_name);
        }
    }
    
    Ok(views)
}

#[tauri::command]
pub async fn get_highlight_profiles(state: State<'_, AppState>) -> Result<Vec<HighlightProfile>, String> {
    state.update_highlight_engine().await?;
    let profiles = state.highlight_store.get_all_profiles().await?;
    Ok(profiles)
}

#[tauri::command]
pub async fn create_highlight_profile(
    mut profile: HighlightProfile,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if profile.is_builtin {
        return Err("Cannot create builtin highlight profile".to_string());
    }
    let now = chrono::Utc::now().timestamp_millis();
    profile.created_at = now;
    profile.updated_at = now;
    state.highlight_store.add_profile(&profile).await?;
    state.update_highlight_engine().await
}

#[tauri::command]
pub async fn update_highlight_profile(
    mut profile: HighlightProfile,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if profile.is_builtin {
        return Err("Cannot update builtin highlight profile".to_string());
    }
    profile.updated_at = chrono::Utc::now().timestamp_millis();
    state.highlight_store.add_profile(&profile).await?;
    state.update_highlight_engine().await
}

#[tauri::command]
pub async fn delete_highlight_profile(
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(p) = state.highlight_store.get_profile(&name).await? {
        if p.is_builtin {
            return Err("Cannot delete builtin highlight profile".to_string());
        }
    }
    state.highlight_store.remove_profile(&name).await?;
    state.update_highlight_engine().await
}

#[tauri::command]
pub async fn export_highlight_profiles(state: State<'_, AppState>) -> Result<String, String> {
    let profiles = state.highlight_store.get_all_profiles().await?;
    serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("Failed to export highlight profiles: {}", e))
}

#[tauri::command]
pub async fn import_highlight_profiles(
    json: String,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let profiles: Vec<HighlightProfile> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse highlight profiles: {}", e))?;
    
    let mut count = 0u32;
    for mut p in profiles {
        p.is_builtin = false;
        p.created_at = chrono::Utc::now().timestamp_millis();
        p.updated_at = p.created_at;
        state.highlight_store.add_profile(&p).await?;
        count += 1;
    }
    
    state.update_highlight_engine().await?;
    Ok(count)
}
