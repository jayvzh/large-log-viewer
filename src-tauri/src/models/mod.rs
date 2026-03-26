use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;

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
    pub source: SmallVec<[u8; 64]>,
    pub message: SmallVec<[u8; 128]>,
    pub raw_offset: u64,
    pub raw_length: u32,
    #[serde(default)]
    pub extra: HashMap<String, String>,
    #[serde(default)]
    pub is_parsed: bool,
}

impl LogEntry {
    pub fn new(
        id: u64,
        file_id: u64,
        line_number: u64,
        timestamp: i64,
        level: LogLevel,
        source: &str,
        message: &str,
        raw_offset: u64,
        raw_length: u32,
    ) -> Self {
        Self {
            id,
            file_id,
            line_number,
            timestamp,
            level,
            source: SmallVec::from_slice(source.as_bytes()),
            message: SmallVec::from_slice(message.as_bytes()),
            raw_offset,
            raw_length,
            extra: HashMap::new(),
            is_parsed: false,
        }
    }
    
    pub fn with_extra(
        id: u64,
        file_id: u64,
        line_number: u64,
        timestamp: i64,
        level: LogLevel,
        source: &str,
        message: &str,
        raw_offset: u64,
        raw_length: u32,
        extra: HashMap<String, String>,
        is_parsed: bool,
    ) -> Self {
        Self {
            id,
            file_id,
            line_number,
            timestamp,
            level,
            source: SmallVec::from_slice(source.as_bytes()),
            message: SmallVec::from_slice(message.as_bytes()),
            raw_offset,
            raw_length,
            extra,
            is_parsed,
        }
    }
    
    pub fn source_str(&self) -> &str {
        std::str::from_utf8(&self.source).unwrap_or("Unknown")
    }
    
    pub fn message_str(&self) -> &str {
        std::str::from_utf8(&self.message).unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryView {
    pub id: u64,
    pub timestamp: i64,
    pub level: String,
    pub source: String,
    pub message: String,
    pub raw: String,
    #[serde(default)]
    pub extra: HashMap<String, String>,
    #[serde(default)]
    pub is_parsed: bool,
    #[serde(default)]
    pub highlight_spans: Vec<HighlightSpan>,
}

impl From<&LogEntry> for LogEntryView {
    fn from(entry: &LogEntry) -> Self {
        Self {
            id: entry.id,
            timestamp: entry.timestamp,
            level: entry.level.as_str().to_string(),
            source: entry.source_str().to_string(),
            message: entry.message_str().to_string(),
            raw: String::new(),
            extra: entry.extra.clone(),
            is_parsed: entry.is_parsed,
            highlight_spans: Vec::new(),
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
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: u64,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub entry_count: u64,
    pub loaded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub encoding: String,
    #[serde(default)]
    pub parallel_workers: Option<usize>,
    #[serde(default = "default_timezone_handling")]
    pub timezone_handling: String,
}

fn default_timezone_handling() -> String {
    "utc".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            encoding: "auto".to_string(),
            parallel_workers: None,
            timezone_handling: default_timezone_handling(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfo {
    pub entries_cleared: u64,
    pub data_dir: String,
    #[serde(default)]
    pub cache_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogTemplate {
    pub name: String,
    pub pattern: String,
    #[serde(rename = "fieldMapping")]
    pub field_mapping: HashMap<String, String>,
    #[serde(rename = "isBuiltin")]
    pub is_builtin: bool,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    #[serde(default, rename = "extraFields")]
    pub extra_fields: Vec<String>,
    #[serde(default, rename = "hasLevel")]
    pub has_level: bool,
    #[serde(default, rename = "hasTimestamp")]
    pub has_timestamp: bool,
    #[serde(default, rename = "hasSource")]
    pub has_source: bool,
    #[serde(default)]
    pub priority: u32,
    #[serde(default, rename = "defaultHighlight")]
    pub default_highlight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogEvent {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateTestResult {
    pub success: bool,
    pub event: Option<LogEvent>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectResult {
    pub template_name: String,
    pub match_rate: f32,
    pub matched_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFilterCondition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub entry_count: u64,
    pub detected_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HighlightStyle {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
}

impl HighlightStyle {
    pub fn to_css_class(&self) -> String {
        let mut classes = Vec::new();
        if let Some(ref color) = self.color {
            classes.push(format!("hl-{}", color.to_lowercase()));
        }
        if self.bold.unwrap_or(false) {
            classes.push("hl-bold".to_string());
        }
        if self.italic.unwrap_or(false) {
            classes.push("hl-italic".to_string());
        }
        if self.underline.unwrap_or(false) {
            classes.push("hl-underline".to_string());
        }
        classes.join(" ")
    }

    pub fn from_str(s: &str) -> Self {
        let mut style = Self::default();
        for part in s.split_whitespace() {
            match part.to_lowercase().as_str() {
                "red" => style.color = Some("red".to_string()),
                "green" => style.color = Some("green".to_string()),
                "blue" => style.color = Some("blue".to_string()),
                "yellow" => style.color = Some("yellow".to_string()),
                "cyan" => style.color = Some("cyan".to_string()),
                "purple" => style.color = Some("purple".to_string()),
                "gray" | "grey" => style.color = Some("gray".to_string()),
                "bold" => style.bold = Some(true),
                "italic" => style.italic = Some(true),
                "underline" => style.underline = Some(true),
                _ => {}
            }
        }
        style
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HighlightRule {
    Field {
        field: String,
        #[serde(default)]
        style_map: HashMap<String, String>,
        #[serde(default)]
        default_style: Option<String>,
    },
    Token {
        pattern: String,
        style: String,
    },
    Keyword {
        words: Vec<String>,
        style: String,
    },
    Regex {
        pattern: String,
        style: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightProfile {
    pub name: String,
    pub rules: Vec<HighlightRule>,
    #[serde(default)]
    pub is_builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
