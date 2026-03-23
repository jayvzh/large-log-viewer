pub mod models;
pub mod database;
pub mod parser;
pub mod reader;
pub mod commands;
pub mod template_store;
pub mod highlight_store;
pub mod highlight_engine;

pub use models::*;
pub use database::Database;
pub use parser::LogParser;
pub use reader::LogFileReader;
pub use template_store::TemplateStore;
pub use highlight_store::HighlightStore;
pub use highlight_engine::HighlightEngine;

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::parser::LogParser;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("FATAL"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("fatal"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("CRITICAL"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("ERR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("WARNING"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("TRACE"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("OTHER"), LogLevel::Other);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Other);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Fatal.as_str(), "FATAL");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Other.as_str(), "OTHER");
    }

    #[test]
    fn test_parse_standard_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:30:45.123 INFO  com.example.Service - 用户登录成功 userId=123";
        let entry = parser.parse_line(line, 1, 0);
        
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source_str(), "com.example.Service");
        assert!(entry.message_str().contains("用户登录成功"));
    }

    #[test]
    fn test_parse_error_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:31:15.456 ERROR com.example.Dao - 数据库连接失败";
        let entry = parser.parse_line(line, 2, 100);
        
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.source_str(), "com.example.Dao");
        assert!(entry.message_str().contains("数据库连接失败"));
    }

    #[test]
    fn test_parse_warn_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:32:00.789 WARN  com.example.Processor - 处理超时";
        let entry = parser.parse_line(line, 3, 200);
        
        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.source_str(), "com.example.Processor");
    }

    #[test]
    fn test_parse_debug_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:33:00.000 DEBUG com.example.Util - 调试信息";
        let entry = parser.parse_line(line, 4, 300);
        
        assert_eq!(entry.level, LogLevel::Debug);
    }

    #[test]
    fn test_parse_trace_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:34:00.000 TRACE com.example.Trace - 跟踪信息";
        let entry = parser.parse_line(line, 5, 400);
        
        assert_eq!(entry.level, LogLevel::Trace);
    }

    #[test]
    fn test_parse_fatal_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:35:00.000 FATAL com.example.App - 致命错误";
        let entry = parser.parse_line(line, 6, 500);
        
        assert_eq!(entry.level, LogLevel::Fatal);
    }

    #[test]
    fn test_parse_unknown_format() {
        let parser = LogParser::new(1);
        let line = "some random text without format";
        let entry = parser.parse_line(line, 7, 600);
        
        assert_eq!(entry.level, LogLevel::Other);
        assert_eq!(entry.source_str(), "Unknown");
    }

    #[test]
    fn test_parse_bracketed_level() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:30:45.123 [ERROR] com.example.Service - 错误信息";
        let entry = parser.parse_line(line, 8, 700);
        
        assert_eq!(entry.level, LogLevel::Error);
    }

    #[test]
    fn test_parse_sample_log_format() {
        let parser = LogParser::new(1);
        let line = "[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Received shutdown signal - ID:5283";
        let entry = parser.parse_line(line, 1, 0);
        
        println!("Level: {:?}", entry.level);
        println!("Source: {}", entry.source_str());
        println!("Message: {}", entry.message_str());
        println!("Timestamp: {}", entry.timestamp);
        
        assert_eq!(entry.level, LogLevel::Fatal);
        assert_eq!(entry.source_str(), "Worker");
        assert!(entry.message_str().contains("Received shutdown signal"));
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_parse_sample_log_with_cr() {
        let parser = LogParser::new(1);
        let line_with_cr = "[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Received shutdown signal - ID:5283\r";
        let entry = parser.parse_line(line_with_cr, 1, 0);
        
        println!("Level with CR: {:?}", entry.level);
        println!("Source with CR: {}", entry.source_str());
        println!("Message with CR: {}", entry.message_str());
        println!("Timestamp with CR: {}", entry.timestamp);
        
        assert_eq!(entry.level, LogLevel::Fatal);
        assert_eq!(entry.source_str(), "Worker");
        assert!(entry.message_str().contains("Received shutdown signal"));
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_decode_line_removes_cr() {
        use crate::reader::{LogFileReader, FileEncoding};
        
        let data_with_cr = b"[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Test message\r";
        let data_without_cr = b"[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Test message";
        
        let line_with_cr = LogFileReader::decode_line(data_with_cr, FileEncoding::Utf8);
        let line_without_cr = LogFileReader::decode_line(data_without_cr, FileEncoding::Utf8);
        
        println!("With CR: '{}'", line_with_cr);
        println!("Without CR: '{}'", line_without_cr);
        
        assert!(!line_with_cr.ends_with('\r'), "Line should not end with CR");
        assert_eq!(line_with_cr, line_without_cr, "Lines should be equal after decode_line");
    }

    #[test]
    fn test_parse_iso_timestamp() {
        let parser = LogParser::new(1);
        let line = "2023-10-01T12:30:45.123 INFO com.example.Service - ISO格式时间戳";
        let entry = parser.parse_line(line, 9, 800);
        
        assert_eq!(entry.level, LogLevel::Info);
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            1,
            100,
            1,
            1696168245000,
            LogLevel::Info,
            "com.example.Test",
            "测试消息",
            0,
            50,
        );
        
        assert_eq!(entry.id, 1);
        assert_eq!(entry.file_id, 100);
        assert_eq!(entry.line_number, 1);
        assert_eq!(entry.timestamp, 1696168245000);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source_str(), "com.example.Test");
        assert_eq!(entry.message_str(), "测试消息");
        assert_eq!(entry.raw_offset, 0);
        assert_eq!(entry.raw_length, 50);
    }

    #[test]
    fn test_log_stats_default() {
        let stats = LogStats::default();
        assert_eq!(stats.all, 0);
        assert_eq!(stats.fatal, 0);
        assert_eq!(stats.error, 0);
        assert_eq!(stats.warn, 0);
        assert_eq!(stats.info, 0);
        assert_eq!(stats.debug, 0);
        assert_eq!(stats.trace, 0);
        assert_eq!(stats.other, 0);
    }

    #[test]
    fn test_parse_progress_serialization() {
        let progress = ParseProgress {
            file_id: 1,
            total_bytes: 1000,
            processed_bytes: 500,
            entries_parsed: 100,
            percentage: 50.0,
            phase: "解析中".to_string(),
            is_complete: false,
        };
        
        let json = serde_json::to_string(&progress).unwrap();
        let parsed: ParseProgress = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.file_id, progress.file_id);
        assert_eq!(parsed.total_bytes, progress.total_bytes);
        assert_eq!(parsed.percentage, progress.percentage);
    }
}

#[cfg(test)]
mod template_tests {
    use crate::models::*;
    use crate::parser::LogTemplateParser;
    use std::collections::HashMap;

    #[test]
    fn test_log_template_creation() {
        let template = LogTemplate {
            name: "Test Template".to_string(),
            pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}) (?P<level>\w+) (?P<message>.+)$".to_string(),
            field_mapping: HashMap::new(),
            is_builtin: false,
            created_at: 0,
            updated_at: 0,
            extra_fields: Vec::new(),
            has_level: true,
            has_timestamp: true,
            has_source: false,
            priority: 0,
            default_highlight: Some("general_default".to_string()),
        };
        
        assert_eq!(template.name, "Test Template");
        assert!(!template.is_builtin);
    }

    #[test]
    fn test_template_parser_standard() {
        let templates = LogTemplateParser::get_builtin_templates();
        let parser = LogTemplateParser::new(templates).unwrap();
        
        let line = "2023-10-01T12:30:45.123+00:00 hostname sshd: Failed password for root from 192.168.1.1 port 22 ssh2";
        let event = parser.parse_line(line);
        
        assert!(event.timestamp.is_some());
        assert_eq!(event.source, Some("sshd".to_string()));
        assert!(event.message.contains("Failed password"));
    }

    #[test]
    fn test_template_parser_field_aliases() {
        let templates = LogTemplateParser::get_builtin_templates();
        let parser = LogTemplateParser::new(templates).unwrap();
        
        let line = "2023-10-01T12:30:45.123+00:00 hostname sshd: Failed password for root from 192.168.1.1 port 22 ssh2";
        let event = parser.parse_line(line);
        
        assert!(event.timestamp.is_some());
    }

    #[test]
    fn test_template_parser_extra_fields() {
        let templates = vec![LogTemplate {
            name: "Extra Fields Test".to_string(),
            pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}) \[(?P<thread_id>\d+)\] (?P<level>\w+) (?P<message>.+)$".to_string(),
            field_mapping: HashMap::new(),
            is_builtin: false,
            created_at: 0,
            updated_at: 0,
            extra_fields: vec!["thread_id".to_string()],
            has_level: true,
            has_timestamp: true,
            has_source: false,
            priority: 0,
            default_highlight: Some("general_default".to_string()),
        }];
        
        let parser = LogTemplateParser::new(templates).unwrap();
        let line = "2023-10-01 [12345] INFO Test message";
        let event = parser.parse_line(line);
        
        assert_eq!(event.extra.get("thread_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_template_parser_no_match() {
        let templates = LogTemplateParser::get_builtin_templates();
        let parser = LogTemplateParser::new(templates).unwrap();
        
        let line = "random text without format";
        let event = parser.parse_line(line);
        
        assert!(event.timestamp.is_none());
        assert!(event.level.is_none());
        assert_eq!(event.message, line);
    }

    #[test]
    fn test_detect_best_template() {
        let templates = LogTemplateParser::get_builtin_templates();
        let parser = LogTemplateParser::new(templates).unwrap();
        
        // Use a line that matches Linux Syslog format
        let lines = vec![
            "2023-10-01T12:30:45.123+00:00 hostname sshd: Failed password for root from 192.168.1.1 port 22 ssh2",
        ];
        
        let refs: Vec<&str> = lines.iter().map(|s| s.as_ref()).collect();
        let results = parser.detect_best_template(&refs);
        
        assert!(!results.is_empty());
        assert_eq!(results[0].template_name, "Linux Syslog");
        assert!(results[0].match_rate > 0.9);
    }

    #[test]
    fn test_test_pattern_success() {
        let pattern = r"^(?P<timestamp>\d{4}-\d{2}-\d{2}) (?P<level>\w+) (?P<message>.+)$";
        let test_line = "2023-10-01 INFO Test message";
        
        let result = LogTemplateParser::test_pattern(pattern, test_line);
        
        assert!(result.success);
        assert!(result.event.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_test_pattern_invalid_regex() {
        let pattern = r"^(?P<timestamp";
        let test_line = "2023-10-01 INFO Test message";
        
        let result = LogTemplateParser::test_pattern(pattern, test_line);
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_test_pattern_no_match() {
        let pattern = r"^(?P<timestamp>\d{4}-\d{2}-\d{2}) (?P<level>\w+)$";
        let test_line = "random text";
        
        let result = LogTemplateParser::test_pattern(pattern, test_line);
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_builtin_templates_count() {
        let templates = LogTemplateParser::get_builtin_templates();
        assert_eq!(templates.len(), 3);
        
        for t in &templates {
            assert!(t.is_builtin);
        }
    }
}
