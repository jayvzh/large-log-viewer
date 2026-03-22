use crate::models::*;
use regex::Regex;
use std::collections::HashMap;

pub struct LogTemplateParser {
    compiled: Vec<(String, Regex)>,
}

impl LogTemplateParser {
    pub fn new(templates: Vec<LogTemplate>) -> Result<Self, String> {
        let mut compiled = Vec::new();
        for t in &templates {
            let re = Regex::new(&t.pattern)
                .map_err(|e| format!("Invalid pattern '{}': {}", t.name, e))?;
            compiled.push((t.name.clone(), re));
        }
        Ok(Self { compiled })
    }
    
    pub fn parse_line(&self, line: &str) -> LogEvent {
        for (name, re) in &self.compiled {
            if let Some(caps) = re.captures(line) {
                return self.captures_to_event(&caps, re, name);
            }
        }
        LogEvent {
            message: line.to_string(),
            ..Default::default()
        }
    }
    
    fn captures_to_event(&self, caps: &regex::Captures, re: &Regex, _template_name: &str) -> LogEvent {
        let mut event = LogEvent::default();
        let mut extra = HashMap::new();
        
        for name in re.capture_names().flatten() {
            if let Some(value) = caps.name(name) {
                let v = value.as_str().to_string();
                match name {
                    "timestamp" | "time" | "ts" => event.timestamp = Some(v),
                    "level" | "lvl" | "severity" => event.level = Some(v),
                    "source" | "logger" | "src" => event.source = Some(v),
                    "message" | "msg" | "content" | "summary" => event.message = v,
                    _ => { extra.insert(name.to_string(), v); }
                }
            }
        }
        
        if event.message.is_empty() {
            if let Some(full) = caps.get(0) {
                event.message = full.as_str().to_string();
            }
        }
        
        event.extra = extra;
        event
    }
    
    pub fn extract_fields_from_pattern(pattern: &str) -> (Vec<String>, bool, bool, bool) {
        let mut extra_fields = Vec::new();
        let mut has_level = false;
        let mut has_timestamp = false;
        let mut has_source = false;
        
        if let Ok(re) = Regex::new(pattern) {
            for name in re.capture_names().flatten() {
                match name {
                    "timestamp" | "time" | "ts" => has_timestamp = true,
                    "level" | "lvl" | "severity" => has_level = true,
                    "source" | "logger" | "src" => has_source = true,
                    "message" | "msg" | "content" | "summary" => {}
                    _ => extra_fields.push(name.to_string()),
                }
            }
        }
        
        (extra_fields, has_level, has_timestamp, has_source)
    }
    
    pub fn get_builtin_templates() -> Vec<LogTemplate> {
        let now = chrono::Utc::now().timestamp_millis();
        vec![
            LogTemplate {
                name: "Standard Format".to_string(),
                pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]\d{3})\s+(?P<level>\w+)\s+(?P<source>\S+)\s+-\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::new(),
                is_builtin: true,
                created_at: now,
                updated_at: now,
                extra_fields: Vec::new(),
                has_level: true,
                has_timestamp: true,
                has_source: true,
            },
            LogTemplate {
                name: "Bracketed Level".to_string(),
                pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]\d{3})\s+\[(?P<level>\w+)\]\s+(?P<source>\S+)\s+-\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::new(),
                is_builtin: true,
                created_at: now,
                updated_at: now,
                extra_fields: Vec::new(),
                has_level: true,
                has_timestamp: true,
                has_source: true,
            },
            LogTemplate {
                name: "Level First".to_string(),
                pattern: r"^(?P<level>\w+)\s*[:\[\]]+\s*(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]?\d*)\s*(?:\[(?P<source>\w+)\])?\s*(?P<message>.+)$".to_string(),
                field_mapping: HashMap::new(),
                is_builtin: true,
                created_at: now,
                updated_at: now,
                extra_fields: Vec::new(),
                has_level: true,
                has_timestamp: true,
                has_source: true,
            },
            LogTemplate {
                name: "Date First".to_string(),
                pattern: r"^(?P<timestamp>\d{2}/\d{2}/\d{4}\s+\d{2}:\d{2}:\d{2})\s*\[(?P<level>\w+)\]\s*(?P<source>\S+)\s*-?\s*(?P<message>.+)$".to_string(),
                field_mapping: HashMap::new(),
                is_builtin: true,
                created_at: now,
                updated_at: now,
                extra_fields: Vec::new(),
                has_level: true,
                has_timestamp: true,
                has_source: true,
            },
            LogTemplate {
                name: "Full Bracketed".to_string(),
                pattern: r"^\[(?P<timestamp>\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}[.,]\d{3})\]\s+\[(?P<level>\w+)\]\s+\[(?P<source>\w+)\](?:\s+\[\w+\])?\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::new(),
                is_builtin: true,
                created_at: now,
                updated_at: now,
                extra_fields: Vec::new(),
                has_level: true,
                has_timestamp: true,
                has_source: true,
            },
        ]
    }
    
    pub fn detect_best_template(&self, lines: &[&str]) -> Vec<DetectResult> {
        let mut results: Vec<DetectResult> = Vec::new();
        
        for (name, re) in &self.compiled {
            let matched = lines.iter().filter(|l| re.is_match(l)).count() as u32;
            let total = lines.len() as u32;
            let rate = if total > 0 { matched as f32 / total as f32 } else { 0.0 };
            
            results.push(DetectResult {
                template_name: name.clone(),
                match_rate: rate,
                matched_count: matched,
                total_count: total,
            });
        }
        
        results.sort_by(|a, b| b.match_rate.partial_cmp(&a.match_rate).unwrap());
        results
    }
    
    pub fn test_pattern(pattern: &str, test_line: &str) -> TemplateTestResult {
        match Regex::new(pattern) {
            Ok(re) => {
                if let Some(caps) = re.captures(test_line) {
                    let mut event = LogEvent::default();
                    let mut extra = HashMap::new();
                    
                    for name in re.capture_names().flatten() {
                        if let Some(value) = caps.name(name) {
                            let v = value.as_str().to_string();
                            match name {
                                "timestamp" | "time" | "ts" => event.timestamp = Some(v),
                                "level" | "lvl" | "severity" => event.level = Some(v),
                                "source" | "logger" | "src" => event.source = Some(v),
                                "message" | "msg" | "content" | "summary" => event.message = v,
                                _ => { extra.insert(name.to_string(), v); }
                            }
                        }
                    }
                    
                    if event.message.is_empty() {
                        if let Some(full) = caps.get(0) {
                            event.message = full.as_str().to_string();
                        }
                    }
                    
                    event.extra = extra;
                    
                    TemplateTestResult {
                        success: true,
                        event: Some(event),
                        error: None,
                    }
                } else {
                    TemplateTestResult {
                        success: false,
                        event: None,
                        error: Some("Pattern does not match the test line".to_string()),
                    }
                }
            }
            Err(e) => TemplateTestResult {
                success: false,
                event: None,
                error: Some(format!("Invalid regex: {}", e)),
            }
        }
    }
}
