use crate::models::*;
use chrono::{NaiveDateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

static LOG_PATTERN_1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]\d{3})\s*(?:\[(?<level>\w+)\])?\s*(?:(?<logger>\S+)\s+)?-?\s*(?<summary>.+)$"
    ).unwrap()
});

static LOG_PATTERN_2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<level>\w+)\s*[:\[\]]+\s*(?<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]?\d*)\s*(?:\[(?<logger>\w+)\])?\s*(?<summary>.+)$"
    ).unwrap()
});

static LOG_PATTERN_3: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<timestamp>\d{2}/\d{2}/\d{4}\s+\d{2}:\d{2}:\d{2})\s*\[(?<level>\w+)\]\s*(?<logger>\S+)\s*-?\s*(?<summary>.+)$"
    ).unwrap()
});

pub struct LogParser {
    file_id: u64,
}

impl LogParser {
    pub fn new(file_id: u64) -> Self {
        Self { file_id }
    }
    
    pub fn parse_line(&self, line: &str, line_number: u64, offset: u64) -> LogEntry {
        let parsed = self.try_parse_standard(line)
            .or_else(|| self.try_parse_level_first(line))
            .or_else(|| self.try_parse_custom(line));
        
        match parsed {
            Some((timestamp, level, logger, summary)) => {
                LogEntry::new(
                    line_number,
                    self.file_id,
                    line_number,
                    timestamp,
                    level,
                    logger,
                    summary,
                    offset,
                    line.len() as u32,
                )
            }
            None => {
                LogEntry::new(
                    line_number,
                    self.file_id,
                    line_number,
                    Utc::now().timestamp_millis(),
                    LogLevel::Other,
                    "Unknown",
                    if line.len() > 200 { &line[..200] } else { line },
                    offset,
                    line.len() as u32,
                )
            }
        }
    }
    
    fn try_parse_standard<'a>(&self, line: &'a str) -> Option<(i64, LogLevel, &'a str, &'a str)> {
        let caps = LOG_PATTERN_1.captures(line)?;
        
        let timestamp = self.parse_timestamp(caps.name("timestamp")?.as_str())?;
        let level = caps.name("level")
            .map(|m| LogLevel::from_str(m.as_str()))
            .unwrap_or(LogLevel::Info);
        let logger = caps.name("logger")
            .map(|m| m.as_str())
            .unwrap_or("Unknown");
        let summary = caps.name("summary")
            .map(|m| m.as_str())
            .unwrap_or("");
        
        Some((timestamp, level, logger, summary))
    }
    
    fn try_parse_level_first<'a>(&self, line: &'a str) -> Option<(i64, LogLevel, &'a str, &'a str)> {
        let caps = LOG_PATTERN_2.captures(line)?;
        
        let level = LogLevel::from_str(caps.name("level")?.as_str());
        let timestamp = self.parse_timestamp(caps.name("timestamp")?.as_str())?;
        let logger = caps.name("logger")
            .map(|m| m.as_str())
            .unwrap_or("Unknown");
        let summary = caps.name("summary")
            .map(|m| m.as_str())
            .unwrap_or("");
        
        Some((timestamp, level, logger, summary))
    }
    
    fn try_parse_custom<'a>(&self, line: &'a str) -> Option<(i64, LogLevel, &'a str, &'a str)> {
        let caps = LOG_PATTERN_3.captures(line)?;
        
        let timestamp = self.parse_timestamp(caps.name("timestamp")?.as_str())?;
        let level = LogLevel::from_str(caps.name("level")?.as_str());
        let logger = caps.name("logger")
            .map(|m| m.as_str())
            .unwrap_or("Unknown");
        let summary = caps.name("summary")
            .map(|m| m.as_str())
            .unwrap_or("");
        
        Some((timestamp, level, logger, summary))
    }
    
    fn parse_timestamp(&self, s: &str) -> Option<i64> {
        let s = s.replace(',', ".");
        
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.3f") {
            return Some(dt.and_utc().timestamp_millis());
        }
        
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f") {
            return Some(dt.and_utc().timestamp_millis());
        }
        
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
            return Some(dt.and_utc().timestamp_millis());
        }
        
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%m/%d/%Y %H:%M:%S") {
            return Some(dt.and_utc().timestamp_millis());
        }
        
        None
    }
    
    pub fn parse_lines_parallel(&self, lines: &[(&str, u64, u64)]) -> Vec<LogEntry> {
        use rayon::prelude::*;
        
        lines.par_iter()
            .map(|(line, line_number, offset)| {
                self.parse_line(line, *line_number, *offset)
            })
            .collect()
    }
}
