use crate::database::Database;
use crate::database::tokenize;
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

pub struct AppState {
    db: Arc<Database>,
    current_file: RwLock<Option<FileInfo>>,
    next_file_id: RwLock<u64>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            current_file: RwLock::new(None),
            next_file_id: RwLock::new(1),
        }
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
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<u64, String> {
    // 获取文件路径，立即释放读锁
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
    
    let reader = LogFileReader::new(path)?;
    let file_size = reader.file_size();
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
    
    for chunk in all_lines.chunks(BATCH_SIZE) {
        let chunk_lines: Vec<(String, u64, u64)> = chunk
            .iter()
            .map(|(data, line_number, offset)| {
                let line_str = String::from_utf8_lossy(data).into_owned();
                (line_str, *line_number, *offset)
            })
            .collect();
        
        let chunk_refs: Vec<(&str, u64, u64)> = chunk_lines
            .iter()
            .map(|(s, ln, off)| (s.as_str(), *ln, *off))
            .collect();
        
        let parsed_entries = parser.parse_lines_parallel(&chunk_refs);
        
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut local_bitmaps: [RoaringBitmap; 7] = Default::default();
        for i in 0..7 {
            local_bitmaps[i] = RoaringBitmap::new();
        }
        
        let mut local_word_index: HashMap<u64, RoaringBitmap> = HashMap::new();
        let mut time_index_batch: Vec<(i64, u64)> = Vec::new();
        
        for mut entry in parsed_entries {
            let current_id = entry_count.fetch_add(1, Ordering::SeqCst);
            entry.id = current_id;
            
            let level_idx = entry.level as usize;
            if level_idx < 7 {
                local_bitmaps[level_idx].insert(current_id as u32);
            }
            
            let text = format!("{} {}", entry.logger_str(), entry.summary_str());
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
            
            batch.push(entry);
        }
        
        state.db.store_entries_batch(&batch).await?;
        
        if !time_index_batch.is_empty() {
            state.db.store_time_index_batch(file_id, &time_index_batch).await?;
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
    
    eprintln!("parse_log: completed, final_count={}, file_id={}", final_count, file_id);
    
    eprintln!("parse_log: storing level bitmaps...");
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
    eprintln!("parse_log: level bitmaps stored");
    
    let _ = app.emit("parse_progress", ParseProgress {
        file_id,
        total_bytes: file_size,
        processed_bytes: file_size,
        entries_parsed: final_count,
        percentage: 95.0,
        phase: "构建搜索索引...".to_string(),
        is_complete: false,
    });
    
    eprintln!("parse_log: building search index, word_index len={}", word_index.len());
    state.db.build_search_index_from_map(file_id, &word_index).await?;
    eprintln!("parse_log: search index built");
    
    let _ = app.emit("parse_progress", ParseProgress {
        file_id,
        total_bytes: file_size,
        processed_bytes: file_size,
        entries_parsed: final_count,
        percentage: 100.0,
        phase: "完成".to_string(),
        is_complete: true,
    });
    
    eprintln!("parse_log: updating current_file...");
    let mut current_file = state.current_file.write().await;
    if let Some(ref mut info) = *current_file {
        info.entry_count = final_count;
    }
    drop(current_file);
    
    eprintln!("parse_log: returning final_count={}", final_count);
    Ok(final_count)
}

#[tauri::command]
pub async fn get_entries(
    file_id: u64,
    offset: u64,
    limit: u64,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntryView>, String> {
    eprintln!("get_entries: file_id={}, offset={}, limit={}", file_id, offset, limit);
    let entries = state.db.get_entries(file_id, offset, limit).await?;
    eprintln!("get_entries: returned {} entries", entries.len());
    
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
    
    Ok(entries.iter().map(LogEntryView::from).collect())
}
