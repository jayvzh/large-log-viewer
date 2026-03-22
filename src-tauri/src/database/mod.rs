use crate::models::*;
use crate::parser::LogTemplateParser;
use roaring::RoaringBitmap;
use sled::Db;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

fn hash_word(word: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    word.hash(&mut hasher);
    hasher.finish()
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}

pub struct Database {
    db: Arc<RwLock<Db>>,
    _data_dir: PathBuf,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let data_dir = std::env::current_exe()
            .map(|p| p.parent().unwrap_or(&PathBuf::from(".")).join("data"))
            .unwrap_or_else(|_| PathBuf::from("data"));
        
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        let pid = std::process::id();
        let db_path = data_dir.join(format!("logs_{}.db", pid));
        let db = sled::open(&db_path)
            .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;
        
        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            _data_dir: data_dir,
        })
    }
    
    pub fn get_data_dir() -> PathBuf {
        std::env::current_exe()
            .map(|p| p.parent().unwrap_or(&PathBuf::from(".")).join("data"))
            .unwrap_or_else(|_| PathBuf::from("data"))
    }
    
    pub async fn clear_cache(&self) -> Result<u64, String> {
        let db = self.db.write().await;
        let count = db.len() as u64;
        let _ = db.clear();
        Ok(count)
    }
    
    pub async fn store_entry(&self, entry: &LogEntry) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("log:{}:{:016x}", entry.file_id, entry.id);
        let value = serde_json::to_vec(entry)
            .map_err(|e| format!("Failed to serialize entry: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store entry: {}", e))?;
        
        Ok(())
    }
    
    pub async fn store_entries_batch(&self, entries: &[LogEntry]) -> Result<(), String> {
        let db = self.db.read().await;
        
        for entry in entries {
            let key = format!("log:{}:{:016x}", entry.file_id, entry.id);
            let value = serde_json::to_vec(entry)
                .map_err(|e| format!("Failed to serialize entry: {}", e))?;
            
            db.insert(key.as_bytes(), value)
                .map_err(|e| format!("Failed to store entry: {}", e))?;
        }
        
        Ok(())
    }
    
    pub async fn get_entry(&self, file_id: u64, entry_id: u64) -> Result<Option<LogEntry>, String> {
        let db = self.db.read().await;
        let key = format!("log:{}:{:016x}", file_id, entry_id);
        
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
    
    pub async fn build_search_index(&self, file_id: u64, entries: &[LogEntry]) -> Result<(), String> {
        let db = self.db.read().await;
        let mut word_index: HashMap<u64, RoaringBitmap> = HashMap::new();
        
        for entry in entries {
            let text = format!("{} {}", entry.source_str(), entry.message_str());
            let words = tokenize(&text);
            for word in words {
                let hash = hash_word(&word);
                word_index
                    .entry(hash)
                    .or_default()
                    .insert(entry.id as u32);
            }
        }
        
        for (hash, bitmap) in word_index {
            let key = format!("word:{}:{:016x}", file_id, hash);
            let mut value = Vec::new();
            bitmap.serialize_into(&mut value)
                .map_err(|e| format!("Failed to serialize bitmap: {}", e))?;
            db.insert(key.as_bytes(), value)
                .map_err(|e| format!("Failed to store word index: {}", e))?;
        }
        
        Ok(())
    }
    
    pub async fn build_search_index_from_map(&self, file_id: u64, word_index: &HashMap<u64, RoaringBitmap>) -> Result<(), String> {
        let mut data_to_write: Vec<(String, Vec<u8>)> = Vec::with_capacity(word_index.len());
        
        for (hash, bitmap) in word_index {
            let key = format!("word:{}:{:016x}", file_id, hash);
            let mut value = Vec::new();
            bitmap.serialize_into(&mut value)
                .map_err(|e| format!("Failed to serialize bitmap: {}", e))?;
            data_to_write.push((key, value));
        }
        
        {
            let db = self.db.read().await;
            for (key, value) in data_to_write {
                db.insert(key.as_bytes(), value)
                    .map_err(|e| format!("Failed to store word index: {}", e))?;
            }
        }
        
        Ok(())
    }
    
    pub async fn get_word_bitmap(&self, file_id: u64, word: &str) -> Result<Option<RoaringBitmap>, String> {
        let db = self.db.read().await;
        let hash = hash_word(word);
        let key = format!("word:{}:{:016x}", file_id, hash);
        
        let value = db.get(key.as_bytes())
            .map_err(|e| format!("Failed to get word bitmap: {}", e))?;
        
        match value {
            Some(v) => {
                let bitmap = RoaringBitmap::deserialize_from(&v[..])
                    .map_err(|e| format!("Failed to deserialize bitmap: {}", e))?;
                Ok(Some(bitmap))
            }
            None => Ok(None),
        }
    }
    
    pub async fn search_with_index(&self, file_id: u64, query: &str, offset: u64, limit: u64) -> Result<Vec<LogEntry>, String> {
        let words = tokenize(query);
        if words.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut result_bitmap = self.get_word_bitmap(file_id, &words[0]).await?
            .unwrap_or_default();
        
        for word in &words[1..] {
            let bitmap = self.get_word_bitmap(file_id, word).await?
                .unwrap_or_default();
            result_bitmap &= bitmap;
        }
        
        let ids: Vec<u64> = result_bitmap.iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|id| id as u64)
            .collect();
        
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entry) = self.get_entry(file_id, id).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    pub async fn store_time_index(&self, file_id: u64, timestamp: i64, entry_id: u64) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("time_idx:{}:{:016x}", file_id, timestamp as u64);
        
        let mut bitmap = match db.get(key.as_bytes()) {
            Ok(Some(v)) => RoaringBitmap::deserialize_from(&v[..])
                .map_err(|e| format!("Failed to deserialize time bitmap: {}", e))?,
            Ok(None) => RoaringBitmap::new(),
            Err(e) => return Err(format!("Failed to get time bitmap: {}", e)),
        };
        
        bitmap.insert(entry_id as u32);
        
        let mut value = Vec::new();
        bitmap.serialize_into(&mut value)
            .map_err(|e| format!("Failed to serialize time bitmap: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store time index: {}", e))?;
        
        Ok(())
    }
    
    pub async fn store_time_index_batch(&self, file_id: u64, entries: &[(i64, u64)]) -> Result<(), String> {
        let db = self.db.read().await;
        let mut time_index: HashMap<i64, RoaringBitmap> = HashMap::new();
        
        for (timestamp, entry_id) in entries {
            time_index
                .entry(*timestamp)
                .or_default()
                .insert(*entry_id as u32);
        }
        
        for (timestamp, bitmap) in time_index {
            let key = format!("time_idx:{}:{:016x}", file_id, timestamp as u64);
            
            let mut merged_bitmap = match db.get(key.as_bytes()) {
                Ok(Some(v)) => RoaringBitmap::deserialize_from(&v[..])
                    .map_err(|e| format!("Failed to deserialize time bitmap: {}", e))?,
                Ok(None) => RoaringBitmap::new(),
                Err(e) => return Err(format!("Failed to get time bitmap: {}", e)),
            };
            
            merged_bitmap |= bitmap;
            
            let mut value = Vec::new();
            merged_bitmap.serialize_into(&mut value)
                .map_err(|e| format!("Failed to serialize time bitmap: {}", e))?;
            
            db.insert(key.as_bytes(), value)
                .map_err(|e| format!("Failed to store time index: {}", e))?;
        }
        
        Ok(())
    }
    
    pub async fn filter_by_time_indexed(&self, file_id: u64, start_time: i64, end_time: i64) -> Result<RoaringBitmap, String> {
        let db = self.db.read().await;
        let start_key = format!("time_idx:{}:{:016x}", file_id, start_time as u64);
        let end_key = format!("time_idx:{}:{:016x}", file_id, end_time as u64);
        
        let mut result = RoaringBitmap::new();
        
        let iter = db.range(start_key.as_bytes()..end_key.as_bytes());
        for item in iter {
            if let Ok((_, value)) = item {
                let bitmap = RoaringBitmap::deserialize_from(&value[..])
                    .map_err(|e| format!("Failed to deserialize time bitmap: {}", e))?;
                result |= bitmap;
            }
        }
        
        Ok(result)
    }
    
    pub async fn get_entries_by_bitmap(&self, file_id: u64, bitmap: &RoaringBitmap, offset: u64, limit: u64) -> Result<Vec<LogEntry>, String> {
        let ids: Vec<u64> = bitmap.iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|id| id as u64)
            .collect();
        
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entry) = self.get_entry(file_id, id).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    pub async fn search_bitmap(&self, file_id: u64, query: &str) -> Result<RoaringBitmap, String> {
        let words = tokenize(query);
        if words.is_empty() {
            return Ok(RoaringBitmap::new());
        }
        
        let mut result = self.get_word_bitmap(file_id, &words[0]).await?
            .unwrap_or_default();
        
        for word in &words[1..] {
            let bitmap = self.get_word_bitmap(file_id, word).await?
                .unwrap_or_default();
            result &= bitmap;
        }
        
        Ok(result)
    }
    
    pub async fn get_entries_by_ids(&self, file_id: u64, ids: &[u64]) -> Result<Vec<LogEntry>, String> {
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entry) = self.get_entry(file_id, *id).await? {
                entries.push(entry);
            }
        }
        Ok(entries)
    }
    
    pub async fn get_all_entries(&self, file_id: u64) -> Result<Vec<LogEntry>, String> {
        let db = self.db.read().await;
        let prefix = format!("log:{}:", file_id);
        
        let mut entries = Vec::new();
        
        let iter = db.scan_prefix(prefix.as_bytes());
        for item in iter {
            if let Ok((_, value)) = item {
                if let Ok(entry) = serde_json::from_slice::<LogEntry>(&value) {
                    entries.push(entry);
                }
            }
        }
        
        entries.sort_by_key(|e| e.id);
        Ok(entries)
    }
    
    pub async fn store_template(&self, template: &LogTemplate) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("template:{}", template.name);
        let value = serde_json::to_vec(template)
            .map_err(|e| format!("Failed to serialize template: {}", e))?;
        
        db.insert(key.as_bytes(), value)
            .map_err(|e| format!("Failed to store template: {}", e))?;
        
        Ok(())
    }

    pub async fn get_template(&self, name: &str) -> Result<Option<LogTemplate>, String> {
        let db = self.db.read().await;
        let key = format!("template:{}", name);
        
        let value = db.get(key.as_bytes())
            .map_err(|e| format!("Failed to get template: {}", e))?;
        
        match value {
            Some(v) => {
                let template: LogTemplate = serde_json::from_slice(&v)
                    .map_err(|e| format!("Failed to deserialize template: {}", e))?;
                Ok(Some(template))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_templates(&self) -> Result<Vec<LogTemplate>, String> {
        let db = self.db.read().await;
        let mut templates = Vec::new();
        
        let iter = db.scan_prefix(b"template:");
        for item in iter {
            if let Ok((_, value)) = item {
                if let Ok(template) = serde_json::from_slice::<LogTemplate>(&value) {
                    templates.push(template);
                }
            }
        }
        
        let mut builtin = LogTemplateParser::get_builtin_templates();
        templates.append(&mut builtin);
        
        Ok(templates)
    }

    pub async fn delete_template(&self, name: &str) -> Result<(), String> {
        let db = self.db.read().await;
        let key = format!("template:{}", name);
        
        db.remove(key.as_bytes())
            .map_err(|e| format!("Failed to delete template: {}", e))?;
        
        Ok(())
    }
}
