use crate::database::{tokenize, Database};
use crate::models::*;
use crate::parser::LogParser;
use crate::reader::LogFileReader;
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
const DETECTION_SAMPLE_LIMIT: usize = 100;
const BATCH_SIZE: usize = 10_000;
const UPDATE_INTERVAL_MS: u64 = 100;

pub struct AppState {
    db: Arc<Database>,
    current_file: RwLock<Option<FileInfo>>,
    next_file_id: RwLock<u64>,
    parse_session: RwLock<Option<ParseSessionInfo>>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            current_file: RwLock::new(None),
            next_file_id: RwLock::new(1),
            parse_session: RwLock::new(None),
        }
    }
}

async fn load_catalog(state: &AppState) -> Result<TemplateCatalog, String> {
    Ok(TemplateCatalog {
        built_in: LogParser::built_in_templates(),
        user_defined: state.db.get_templates().await?,
    })
}

fn select_templates(
    catalog: &TemplateCatalog,
    template_name: Option<&str>,
) -> Result<Vec<StoredLogTemplate>, String> {
    let mut all = catalog.built_in.clone();
    all.extend(catalog.user_defined.clone());

    match template_name {
        Some(name) if !name.is_empty() && name != "auto-detect" => all
            .into_iter()
            .find(|template| template.name == name)
            .map(|template| vec![template])
            .ok_or_else(|| format!("Template not found: {}", name)),
        _ => Ok(all),
    }
}

fn collect_sample_lines(path: &PathBuf, limit: usize) -> Result<Vec<String>, String> {
    let reader = LogFileReader::new(path.clone())?;
    let mut iterator = reader.read_lines()?;
    let mut lines = Vec::with_capacity(limit);

    while lines.len() < limit {
        match iterator.next() {
            Some(Ok((line, _, _))) => lines.push(line),
            Some(Err(error)) => return Err(format!("Failed to read sample lines: {}", error)),
            None => break,
        }
    }

    Ok(lines)
}

#[tauri::command]
pub async fn get_templates(state: State<'_, AppState>) -> Result<TemplateCatalog, String> {
    load_catalog(&state).await
}

#[tauri::command]
pub async fn save_template(
    template: StoredLogTemplate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    LogParser::compile_template(&template)?;
    state.db.store_template(&template).await
}

#[tauri::command]
pub async fn delete_template(
    template_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.delete_template(&template_name).await
}

#[tauri::command]
pub async fn preview_template(
    template: StoredLogTemplate,
    sample_line: String,
) -> Result<TemplatePreviewResponse, String> {
    Ok(LogParser::preview_template(&template, &sample_line))
}

#[tauri::command]
pub async fn detect_template_for_lines(
    lines: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<TemplateMatchSummary>, String> {
    let catalog = load_catalog(&state).await?;
    let templates = select_templates(&catalog, None)?;
    let parser = LogParser::new(0, LogParser::compile_templates(&templates)?);
    Ok(parser.detect_template(&lines))
}

#[tauri::command]
pub async fn get_parse_session(
    state: State<'_, AppState>,
) -> Result<Option<ParseSessionInfo>, String> {
    Ok(state.parse_session.read().await.clone())
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

    let metadata =
        std::fs::metadata(&path_buf).map_err(|e| format!("Failed to get file metadata: {}", e))?;
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
    *state.current_file.write().await = Some(file_info.clone());
    *state.parse_session.write().await = None;

    Ok(file_info)
}

#[tauri::command]
pub async fn parse_log(
    file_id: u64,
    template_name: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<u64, String> {
    let path = {
        let current_file = state.current_file.read().await;
        let file_info = current_file.as_ref().ok_or("No file loaded")?;
        PathBuf::from(&file_info.path)
    };

    let settings = get_settings().await.unwrap_or_default();
    if let Some(workers) = settings.parallel_workers {
        if workers > 0 {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build_global();
        }
    }

    let catalog = load_catalog(&state).await?;
    let selection = select_templates(&catalog, template_name.as_deref())?;
    let detection_input = collect_sample_lines(&path, DETECTION_SAMPLE_LIMIT)?;

    let active_templates = if matches!(template_name.as_deref(), None | Some("auto-detect")) {
        let detection_parser = LogParser::new(file_id, LogParser::compile_templates(&selection)?);
        let detection = detection_parser.detect_template(&detection_input);
        let best = detection
            .first()
            .map(|summary| summary.template_name.clone());
        let mut ordered = selection.clone();
        if let Some(best_name) = best {
            ordered.sort_by_key(|template| if template.name == best_name { 0 } else { 1 });
        }
        *state.parse_session.write().await = Some(ParseSessionInfo {
            mode: "auto-detect".to_string(),
            active_template: best.unwrap_or_else(|| "unparsed".to_string()),
            detection,
        });
        ordered
    } else {
        *state.parse_session.write().await = Some(ParseSessionInfo {
            mode: "manual".to_string(),
            active_template: selection[0].name.clone(),
            detection: Vec::new(),
        });
        selection
    };

    let parser = LogParser::new(file_id, LogParser::compile_templates(&active_templates)?);
    let reader = LogFileReader::new(path)?;
    let file_size = reader.file_size();
    let mut iterator = reader.read_lines()?;

    let mut level_bitmaps: [RoaringBitmap; 7] = Default::default();
    for bitmap in &mut level_bitmaps {
        *bitmap = RoaringBitmap::new();
    }

    let mut word_index: HashMap<u64, RoaringBitmap> = HashMap::new();
    let entry_count = AtomicU64::new(0);
    let mut last_update = Instant::now();
    let mut chunk: Vec<(String, u64, u64)> = Vec::with_capacity(BATCH_SIZE);
    let mut last_processed_bytes = 0_u64;

    let session = state.parse_session.read().await.clone();
    let phase = session
        .as_ref()
        .map(|info| format!("使用模板: {}", info.active_template))
        .unwrap_or_else(|| "解析中".to_string());

    let _ = app.emit(
        "parse_progress",
        ParseProgress {
            file_id,
            total_bytes: file_size,
            processed_bytes: 0,
            entries_parsed: 0,
            percentage: 0.0,
            phase,
            is_complete: false,
        },
    );

    loop {
        let next = iterator.next();
        match next {
            Some(Ok((line, line_number, offset))) => {
                last_processed_bytes = offset + line.len() as u64;
                chunk.push((line, line_number, offset));
                if chunk.len() >= BATCH_SIZE {
                    process_chunk(
                        file_id,
                        &parser,
                        &state,
                        &entry_count,
                        &mut word_index,
                        &mut level_bitmaps,
                        &mut chunk,
                    )
                    .await?;
                }
            }
            Some(Err(error)) => return Err(format!("Failed to read log file: {}", error)),
            None => {
                if !chunk.is_empty() {
                    process_chunk(
                        file_id,
                        &parser,
                        &state,
                        &entry_count,
                        &mut word_index,
                        &mut level_bitmaps,
                        &mut chunk,
                    )
                    .await?;
                }
                break;
            }
        }

        if last_update.elapsed().as_millis() as u64 >= UPDATE_INTERVAL_MS {
            last_update = Instant::now();
            let percentage = if file_size == 0 {
                100.0
            } else {
                (last_processed_bytes as f32 / file_size as f32) * 100.0
            };
            let _ = app.emit(
                "parse_progress",
                ParseProgress {
                    file_id,
                    total_bytes: file_size,
                    processed_bytes: last_processed_bytes.min(file_size),
                    entries_parsed: entry_count.load(Ordering::SeqCst),
                    percentage,
                    phase: "流式解析日志中...".to_string(),
                    is_complete: false,
                },
            );
        }
    }

    for (index, bitmap) in level_bitmaps.iter().enumerate() {
        let level = match index {
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

    let _ = app.emit(
        "parse_progress",
        ParseProgress {
            file_id,
            total_bytes: file_size,
            processed_bytes: file_size,
            entries_parsed: entry_count.load(Ordering::SeqCst),
            percentage: 97.0,
            phase: "构建搜索索引...".to_string(),
            is_complete: false,
        },
    );

    state
        .db
        .build_search_index_from_map(file_id, &word_index)
        .await?;
    let final_count = entry_count.load(Ordering::SeqCst);

    let _ = app.emit(
        "parse_progress",
        ParseProgress {
            file_id,
            total_bytes: file_size,
            processed_bytes: file_size,
            entries_parsed: final_count,
            percentage: 100.0,
            phase: "完成".to_string(),
            is_complete: true,
        },
    );

    let mut current_file = state.current_file.write().await;
    if let Some(info) = current_file.as_mut() {
        info.entry_count = final_count;
    }

    Ok(final_count)
}

async fn process_chunk(
    file_id: u64,
    parser: &LogParser,
    state: &AppState,
    entry_count: &AtomicU64,
    word_index: &mut HashMap<u64, RoaringBitmap>,
    level_bitmaps: &mut [RoaringBitmap; 7],
    chunk: &mut Vec<(String, u64, u64)>,
) -> Result<(), String> {
    let chunk_refs: Vec<(&str, u64, u64)> = chunk
        .iter()
        .map(|(line, number, offset)| (line.as_str(), *number, *offset))
        .collect();
    let parsed_entries = parser.parse_lines_parallel(&chunk_refs);
    let mut batch = Vec::with_capacity(parsed_entries.len());
    let mut time_index_batch: Vec<(i64, u64)> = Vec::new();

    for mut entry in parsed_entries {
        let current_id = entry_count.fetch_add(1, Ordering::SeqCst);
        entry.id = current_id;
        let level_idx = entry.level as usize;
        if level_idx < 7 {
            level_bitmaps[level_idx].insert(current_id as u32);
        }

        let text = format!(
            "{} {} {}",
            entry.logger_str(),
            entry.summary_str(),
            entry.extra.values().cloned().collect::<Vec<_>>().join(" ")
        );
        for word in tokenize(&text) {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            word.hash(&mut hasher);
            let hash = hasher.finish();
            word_index
                .entry(hash)
                .or_default()
                .insert(current_id as u32);
        }

        if entry.timestamp > 0 {
            time_index_batch.push((entry.timestamp, current_id));
        }

        batch.push(entry);
    }

    state.db.store_entries_batch(&batch).await?;
    if !time_index_batch.is_empty() {
        state
            .db
            .store_time_index_batch(file_id, &time_index_batch)
            .await?;
    }

    chunk.clear();
    Ok(())
}

#[tauri::command]
pub async fn get_entries(
    file_id: u64,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let entries = state.db.get_entries(file_id, offset, limit).await?;
    Ok(entries.iter().map(LogEntryView::from).collect())
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
    state: State<'_, AppState>,
) -> Result<Option<LogEntryView>, String> {
    let entry = state.db.get_entry(file_id, entry_id).await?;
    match entry {
        Some(e) => {
            let file_path = {
                let current_file = state.current_file.read().await;
                current_file.as_ref().map(|f| PathBuf::from(&f.path))
            };
            let raw = if let Some(path) = file_path {
                let reader = LogFileReader::new(path)?;
                reader.read_raw_content(e.raw_offset, e.raw_length)?
            } else {
                String::new()
            };
            let mut view = LogEntryView::from(&e);
            view.raw = raw;
            Ok(Some(view))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_stats(file_id: u64, state: State<'_, AppState>) -> Result<LogStats, String> {
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
    let bitmap = state
        .db
        .get_level_bitmap(file_id, level)
        .await?
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
    let entries = state
        .db
        .search_with_index(file_id, &query, offset, limit)
        .await?;
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
    let bitmap = state
        .db
        .filter_by_time_indexed(file_id, start_time, end_time)
        .await?;
    let entries = state
        .db
        .get_entries_by_bitmap(file_id, &bitmap, offset, limit)
        .await?;
    Ok(entries.iter().map(LogEntryView::from).collect())
}

#[tauri::command]
pub async fn get_current_file(state: State<'_, AppState>) -> Result<Option<FileInfo>, String> {
    Ok(state.current_file.read().await.clone())
}

#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> Result<CacheInfo, String> {
    let count = state.db.clear_cache().await?;
    *state.current_file.write().await = None;
    *state.parse_session.write().await = None;
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
        std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0)
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
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;
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
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    let mut result_bitmap: Option<RoaringBitmap> = None;

    if let Some(level_value) = level {
        let level_enum = LogLevel::from_str(&level_value);
        let level_bitmap = state
            .db
            .get_level_bitmap(file_id, level_enum)
            .await?
            .unwrap_or_default();
        result_bitmap = Some(level_bitmap);
    }

    if let (Some(start), Some(end)) = (start_time, end_time) {
        let time_bitmap = state.db.filter_by_time_indexed(file_id, start, end).await?;
        result_bitmap = match result_bitmap {
            Some(bitmap) => Some(bitmap & time_bitmap),
            None => Some(time_bitmap),
        };
    }

    if let Some(query_value) = query {
        let search_bitmap = state.db.search_bitmap(file_id, &query_value).await?;
        result_bitmap = match result_bitmap {
            Some(bitmap) => Some(bitmap & search_bitmap),
            None => Some(search_bitmap),
        };
    }

    let entries = match result_bitmap {
        Some(bitmap) => {
            let ids: Vec<u64> = bitmap
                .iter()
                .skip(offset as usize)
                .take(limit as usize)
                .map(|id| id as u64)
                .collect();
            state.db.get_entries_by_ids(file_id, &ids).await?
        }
        None => state.db.get_entries(file_id, offset, limit).await?,
    };

    Ok(entries.iter().map(LogEntryView::from).collect())
}
