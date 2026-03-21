pub mod models;
pub mod database;
pub mod parser;
pub mod reader;
pub mod commands;

pub use models::*;
pub use database::Database;
pub use parser::LogParser;
pub use reader::LogFileReader;

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
        assert_eq!(entry.logger_str(), "com.example.Service");
        assert!(entry.summary_str().contains("用户登录成功"));
    }

    #[test]
    fn test_parse_error_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:31:15.456 ERROR com.example.Dao - 数据库连接失败";
        let entry = parser.parse_line(line, 2, 100);
        
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.logger_str(), "com.example.Dao");
        assert!(entry.summary_str().contains("数据库连接失败"));
    }

    #[test]
    fn test_parse_warn_log() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:32:00.789 WARN  com.example.Processor - 处理超时";
        let entry = parser.parse_line(line, 3, 200);
        
        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.logger_str(), "com.example.Processor");
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
        assert_eq!(entry.logger_str(), "Unknown");
    }

    #[test]
    fn test_parse_bracketed_level() {
        let parser = LogParser::new(1);
        let line = "2023-10-01 12:30:45.123 [ERROR] com.example.Service - 错误信息";
        let entry = parser.parse_line(line, 8, 700);
        
        assert_eq!(entry.level, LogLevel::Error);
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
        assert_eq!(entry.logger_str(), "com.example.Test");
        assert_eq!(entry.summary_str(), "测试消息");
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
