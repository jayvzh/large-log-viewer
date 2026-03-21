use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Fatal = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
    Other = 6,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FATAL" | "CRITICAL" => LogLevel::Fatal,
            "ERROR" | "ERR" | "SEVERE" => LogLevel::Error,
            "WARN" | "WARNING" => LogLevel::Warn,
            "INFO" | "INFORMATION" => LogLevel::Info,
            "DEBUG" | "DBG" => LogLevel::Debug,
            "TRACE" | "TRC" => LogLevel::Trace,
            _ => LogLevel::Other,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Fatal => "FATAL",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
            LogLevel::Other => "OTHER",
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub file_id: u64,
    pub line_number: u64,
    pub timestamp: i64,
    pub level: LogLevel,
    pub logger: SmallVec<[u8; 64]>,
    pub summary: SmallVec<[u8; 128]>,
    pub raw_offset: u64,
    pub raw_length: u32,
}

impl LogEntry {
    pub fn new(
        id: u64,
        file_id: u64,
        line_number: u64,
        timestamp: i64,
        level: LogLevel,
        logger: &str,
        summary: &str,
        raw_offset: u64,
        raw_length: u32,
    ) -> Self {
        Self {
            id,
            file_id,
            line_number,
            timestamp,
            level,
            logger: SmallVec::from_slice(logger.as_bytes()),
            summary: SmallVec::from_slice(summary.as_bytes()),
            raw_offset,
            raw_length,
        }
    }
    
    pub fn logger_str(&self) -> &str {
        std::str::from_utf8(&self.logger).unwrap_or("Unknown")
    }
    
    pub fn summary_str(&self) -> &str {
        std::str::from_utf8(&self.summary).unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryView {
    pub id: u64,
    pub timestamp: i64,
    pub level: String,
    pub logger: String,
    pub summary: String,
    pub raw: String,
}

impl From<&LogEntry> for LogEntryView {
    fn from(entry: &LogEntry) -> Self {
        Self {
            id: entry.id,
            timestamp: entry.timestamp,
            level: entry.level.as_str().to_string(),
            logger: entry.logger_str().to_string(),
            summary: entry.summary_str().to_string(),
            raw: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogStats {
    pub all: u64,
    pub fatal: u64,
    pub error: u64,
    pub warn: u64,
    pub info: u64,
    pub debug: u64,
    pub trace: u64,
    pub other: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: u64,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub entry_count: u64,
    pub loaded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseProgress {
    pub file_id: u64,
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub entries_parsed: u64,
    pub percentage: f32,
    pub phase: String,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub encoding: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            encoding: "auto".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub entries_cleared: u64,
    pub data_dir: String,
    #[serde(default)]
    pub cache_size: u64,
}
