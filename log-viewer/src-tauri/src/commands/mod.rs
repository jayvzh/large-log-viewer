use crate::database::Database;
use crate::models::*;
use crate::parser::LogParser;
use crate::reader::LogFileReader;
use roaring::RoaringBitmap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

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
    
    let mut current_file = state.current_file.write().await;
    *current_file = Some(file_info.clone());
    
    Ok(file_info)
}

#[tauri::command]
pub async fn parse_log(
    file_id: u64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<u64, String> {
    let file_info = state.current_file.read().await;
    let file_info = file_info.as_ref()
        .ok_or("No file loaded")?
        .clone();
    drop(file_info);
    
    let path = PathBuf::from(
        state.current_file.read().await.as_ref().unwrap().path.clone()
    );
    
    let reader = LogFileReader::new(path)?;
    let file_size = reader.file_size();
    let parser = LogParser::new(file_id);
    
    let mut level_bitmaps: [RoaringBitmap; 7] = Default::default();
    for i in 0..7 {
        level_bitmaps[i] = RoaringBitmap::new();
    }
    
    let mut entry_count = 0u64;
    let mut batch = Vec::with_capacity(1000);
    let mut last_progress = 0.0f32;
    
    let lines = reader.read_lines_mmap()?;
    
    for line in lines {
        let line_str = String::from_utf8_lossy(&line.data);
        let line_str = line_str.as_ref();
        
        let mut entry = parser.parse_line(line_str, line.line_number, line.offset);
        entry.id = entry_count;
        
        let level_idx = entry.level as usize;
        if level_idx < 7 {
            level_bitmaps[level_idx].insert(entry_count as u32);
        }
        
        batch.push(entry);
        entry_count += 1;
        
        if batch.len() >= 1000 {
            state.db.store_entries_batch(&batch).await?;
            batch.clear();
        }
        
        let progress = (line.offset as f32 / file_size as f32) * 100.0;
        if progress - last_progress >= 1.0 {
            last_progress = progress;
            
            let _ = app.emit("parse_progress", ParseProgress {
                file_id,
                total_bytes: file_size,
                processed_bytes: line.offset,
                entries_parsed: entry_count,
                percentage: progress,
                is_complete: false,
            });
        }
    }
    
    if !batch.is_empty() {
        state.db.store_entries_batch(&batch).await?;
    }
    
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
        entries_parsed: entry_count,
        percentage: 100.0,
        is_complete: true,
    });
    
    let mut current_file = state.current_file.write().await;
    if let Some(ref mut info) = *current_file {
        info.entry_count = entry_count;
    }
    
    Ok(entry_count)
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
pub async fn get_entry_detail(
    file_id: u64,
    entry_id: u64,
    state: State<'_, AppState>,
) -> Result<Option<LogEntryView>, String> {
    let entry = state.db.get_entry(file_id, entry_id).await?;
    
    Ok(entry.as_ref().map(LogEntryView::from))
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
    let entries = state.db.get_entries(file_id, 0, 10000).await?;
    
    let query_lower = query.to_lowercase();
    let filtered: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.logger_str().to_lowercase().contains(&query_lower) ||
            e.summary_str().to_lowercase().contains(&query_lower)
        })
        .skip(offset as usize)
        .take(limit as usize)
        .map(LogEntryView::from)
        .collect();
    
    Ok(filtered)
}

#[tauri::command]
pub async fn get_current_file(state: State<'_, AppState>) -> Result<Option<FileInfo>, String> {
    let file = state.current_file.read().await;
    Ok(file.clone())
}
