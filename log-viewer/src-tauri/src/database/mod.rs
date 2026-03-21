use crate::models::*;
use roaring::RoaringBitmap;
use sled::Db;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Database {
    db: Arc<RwLock<Db>>,
    data_dir: PathBuf,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("LogViewer")
            .join("data");
        
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        let db_path = data_dir.join("logs.db");
        let db = sled::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            data_dir,
        })
    }
    
    pub async fn store_entry(&self, entry: &LogEntry) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("log:{}:{}", entry.file_id, entry.id);
        let value = serde_json::to_vec(entry)
            .map_err(|e| format!("Failed to serialize entry: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store entry: {}", e))?;
        
        Ok(())
    }
    
    pub async fn store_entries_batch(&self, entries: &[LogEntry]) -> Result<(), String> {
        let db = self.db.read().await;
        
        for entry in entries {
            let key = format!("log:{}:{}", entry.file_id, entry.id);
            let value = serde_json::to_vec(entry)
                .map_err(|e| format!("Failed to serialize entry: {}", e))?;
            
            db.insert(key.as_bytes(), value)
                .map_err(|e| format!("Failed to store entry: {}", e))?;
        }
        
        Ok(())
    }
    
    pub async fn get_entry(&self, file_id: u64, entry_id: u64) -> Result<Option<LogEntry>, String> {
        let db = self.db.read().await;
        let key = format!("log:{}:{}", file_id, entry_id);
        
        let value = db.get(key.as_bytes())
            .map_err(|e| format!("Failed to get entry: {}", e))?;
        
        match value {
            Some(v) => {
                let entry: LogEntry = serde_json::from_slice(&v)
                    .map_err(|e| format!("Failed to deserialize entry: {}", e))?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }
    
    pub async fn get_entries(&self, file_id: u64, offset: u64, limit: u64) -> Result<Vec<LogEntry>, String> {
        let db = self.db.read().await;
        let mut entries = Vec::with_capacity(limit as usize);
        let prefix = format!("log:{}:", file_id);
        
        let iter = db.scan_prefix(prefix.as_bytes());
        let mut count: u64 = 0;
        
        for item in iter {
            if count >= offset + limit {
                break;
            }
            if let Ok((_, value)) = item {
                if count >= offset {
                    let entry: LogEntry = serde_json::from_slice(&value)
                        .map_err(|e| format!("Failed to deserialize entry: {}", e))?;
                    entries.push(entry);
                }
                count += 1;
            }
        }
        
        Ok(entries)
    }
    
    pub async fn store_level_bitmap(&self, file_id: u64, level: LogLevel, bitmap: &RoaringBitmap) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("level_bitmap:{}:{}", file_id, level as u8);
        let mut value = Vec::new();
        bitmap.serialize_into(&mut value)
            .map_err(|e| format!("Failed to serialize bitmap: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store bitmap: {}", e))?;
        
        Ok(())
    }
    
    pub async fn get_level_bitmap(&self, file_id: u64, level: LogLevel) -> Result<Option<RoaringBitmap>, String> {
        let db = self.db.read().await;
        let key = format!("level_bitmap:{}:{}", file_id, level as u8);
        
        let value = db.get(key.as_bytes())
            .map_err(|e| format!("Failed to get bitmap: {}", e))?;
        
        match value {
            Some(v) => {
                let bitmap = RoaringBitmap::deserialize_from(&v[..])
                    .map_err(|e| format!("Failed to deserialize bitmap: {}", e))?;
                Ok(Some(bitmap))
            }
            None => Ok(None),
        }
    }
    
    pub async fn store_file_info(&self, file_info: &FileInfo) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("file:{}", file_info.id);
        let value = serde_json::to_vec(file_info)
            .map_err(|e| format!("Failed to serialize file info: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store file info: {}", e))?;
        
        Ok(())
    }
    
    pub async fn get_stats(&self, file_id: u64) -> Result<LogStats, String> {
        let mut stats = LogStats::default();
        
        for level in [LogLevel::Fatal, LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug, LogLevel::Trace, LogLevel::Other] {
            if let Some(bitmap) = self.get_level_bitmap(file_id, level).await? {
                let count = bitmap.len() as u64;
                match level {
                    LogLevel::Fatal => stats.fatal = count,
                    LogLevel::Error => stats.error = count,
                    LogLevel::Warn => stats.warn = count,
                    LogLevel::Info => stats.info = count,
                    LogLevel::Debug => stats.debug = count,
                    LogLevel::Trace => stats.trace = count,
                    LogLevel::Other => stats.other = count,
                }
            }
        }
        
        stats.all = stats.fatal + stats.error + stats.warn + stats.info + stats.debug + stats.trace + stats.other;
        
        Ok(stats)
    }
}
