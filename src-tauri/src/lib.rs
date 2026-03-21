pub mod commands;
pub mod database;
pub mod models;
pub mod parser;
pub mod reader;

pub use database::Database;
pub use models::*;
pub use parser::LogParser;
pub use reader::LogFileReader;

#[cfg(test)]
mod tests {
    use crate::models::{LogLevel, StoredLogTemplate};
    use crate::parser::LogParser;
    use std::collections::HashMap;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("FATAL"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("TRACE"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Other);
    }

    #[test]
    fn test_named_capture_template_preview() {
        let template = StoredLogTemplate {
            name: "preview-template".to_string(),
            pattern: r"^(?P<ts>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (?P<sev>\w+) (?P<src>\S+) - (?P<body>.+) user=(?P<user>\w+)$".to_string(),
            field_mapping: HashMap::from([
                ("timestamp".to_string(), "ts".to_string()),
                ("level".to_string(), "sev".to_string()),
                ("source".to_string(), "src".to_string()),
                ("message".to_string(), "body".to_string()),
            ]),
        };

        let preview = LogParser::preview_template(
            &template,
            "2024-01-02 03:04:05 INFO auth.service - login ok user=alice",
        );
        assert!(preview.matched);
        let event = preview.event.expect("preview should produce event");
        assert_eq!(event.timestamp.as_deref(), Some("2024-01-02 03:04:05"));
        assert_eq!(event.level.as_deref(), Some("INFO"));
        assert_eq!(event.source.as_deref(), Some("auth.service"));
        assert_eq!(event.message, "login ok");
        assert_eq!(event.extra.get("user").map(String::as_str), Some("alice"));
    }

    #[test]
    fn test_auto_detect_selects_best_template() {
        let templates = LogParser::built_in_templates();
        let parser = LogParser::new(1, LogParser::compile_templates(&templates).unwrap());
        let sample = vec![
            "2023-10-01 12:30:45.123 INFO svc.Auth - login ok".to_string(),
            "2023-10-01 12:31:45.123 ERROR svc.Auth - login failed".to_string(),
        ];

        let detection = parser.detect_template(&sample);
        assert_eq!(
            detection.first().map(|s| s.template_name.as_str()),
            Some("default-standard")
        );
        assert!(detection.first().unwrap().success_rate > 0.9);
    }

    #[test]
    fn test_parse_line_uses_template_system() {
        let templates = LogParser::built_in_templates();
        let parser = LogParser::new(42, LogParser::compile_templates(&templates).unwrap());
        let entry = parser.parse_line("2023-10-01 12:30:45.123 INFO svc.Auth - login ok", 7, 128);

        assert_eq!(entry.file_id, 42);
        assert_eq!(entry.line_number, 7);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.logger_str(), "svc.Auth");
        assert_eq!(entry.summary_str(), "login ok");
        assert_eq!(entry.template_name, "default-standard");
    }
}
