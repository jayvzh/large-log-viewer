use crate::models::{
    LogEntry, LogEvent, LogLevel, StoredLogTemplate, TemplateMatchSummary, TemplatePreviewResponse,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LogTemplate {
    pub name: String,
    pub pattern: Regex,
    pub field_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TemplateParseResult {
    pub template_name: String,
    pub event: LogEvent,
}

pub struct LogParser {
    file_id: u64,
    templates: Vec<LogTemplate>,
    fallback_template_name: String,
}

impl LogParser {
    pub fn new(file_id: u64, templates: Vec<LogTemplate>) -> Self {
        Self {
            file_id,
            templates,
            fallback_template_name: "unparsed".to_string(),
        }
    }

    pub fn compile_template(template: &StoredLogTemplate) -> Result<LogTemplate, String> {
        if template.name.trim().is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        let pattern = Regex::new(&template.pattern)
            .map_err(|error| format!("Failed to compile template {}: {}", template.name, error))?;
        let capture_names: std::collections::HashSet<String> = pattern
            .capture_names()
            .flatten()
            .map(str::to_string)
            .collect();

        if !template.field_mapping.contains_key("message") {
            return Err(format!(
                "Template {} must map the message field",
                template.name
            ));
        }

        for (standard_field, capture_name) in &template.field_mapping {
            if !capture_names.contains(capture_name) {
                return Err(format!(
                    "Template {} maps {} to missing capture group {}",
                    template.name, standard_field, capture_name
                ));
            }
        }

        Ok(LogTemplate {
            name: template.name.clone(),
            pattern,
            field_mapping: template.field_mapping.clone(),
        })
    }

    pub fn built_in_templates() -> Vec<StoredLogTemplate> {
        vec![
            StoredLogTemplate {
                name: "default-standard".to_string(),
                pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]?\d*)\s+(?P<level>\w+)\s+(?P<logger>\S+)\s+-\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::from([
                    ("timestamp".to_string(), "timestamp".to_string()),
                    ("level".to_string(), "level".to_string()),
                    ("source".to_string(), "logger".to_string()),
                    ("message".to_string(), "message".to_string()),
                ]),
            },
            StoredLogTemplate {
                name: "default-bracketed-level".to_string(),
                pattern: r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]?\d*)\s+\[(?P<level>\w+)\]\s+(?P<logger>\S+)\s+-\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::from([
                    ("timestamp".to_string(), "timestamp".to_string()),
                    ("level".to_string(), "level".to_string()),
                    ("source".to_string(), "logger".to_string()),
                    ("message".to_string(), "message".to_string()),
                ]),
            },
            StoredLogTemplate {
                name: "default-level-first".to_string(),
                pattern: r"^(?P<level>\w+)\s*[:\[\]]+\s*(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[.,]?\d*)\s*(?:\[(?P<logger>[^\]]+)\])?\s*(?P<message>.+)$".to_string(),
                field_mapping: HashMap::from([
                    ("timestamp".to_string(), "timestamp".to_string()),
                    ("level".to_string(), "level".to_string()),
                    ("source".to_string(), "logger".to_string()),
                    ("message".to_string(), "message".to_string()),
                ]),
            },
            StoredLogTemplate {
                name: "default-slash-date".to_string(),
                pattern: r"^(?P<timestamp>\d{2}/\d{2}/\d{4}\s+\d{2}:\d{2}:\d{2})\s*\[(?P<level>\w+)\]\s*(?P<logger>\S+)\s*-?\s*(?P<message>.+)$".to_string(),
                field_mapping: HashMap::from([
                    ("timestamp".to_string(), "timestamp".to_string()),
                    ("level".to_string(), "level".to_string()),
                    ("source".to_string(), "logger".to_string()),
                    ("message".to_string(), "message".to_string()),
                ]),
            },
            StoredLogTemplate {
                name: "default-fully-bracketed".to_string(),
                pattern: r"^\[(?P<timestamp>\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}[.,]?\d*)\]\s+\[(?P<level>\w+)\]\s+\[(?P<logger>[^\]]+)\](?:\s+\[[^\]]+\])?\s+(?P<message>.+)$".to_string(),
                field_mapping: HashMap::from([
                    ("timestamp".to_string(), "timestamp".to_string()),
                    ("level".to_string(), "level".to_string()),
                    ("source".to_string(), "logger".to_string()),
                    ("message".to_string(), "message".to_string()),
                ]),
            },
        ]
    }

    pub fn compile_templates(templates: &[StoredLogTemplate]) -> Result<Vec<LogTemplate>, String> {
        templates.iter().map(Self::compile_template).collect()
    }

    pub fn preview_template(template: &StoredLogTemplate, sample: &str) -> TemplatePreviewResponse {
        match Self::compile_template(template) {
            Ok(compiled) => match Self::parse_with_template(sample, &compiled) {
                Some(event) => TemplatePreviewResponse {
                    matched: true,
                    template_name: compiled.name,
                    event: Some(event),
                    error: None,
                },
                None => TemplatePreviewResponse {
                    matched: false,
                    template_name: template.name.clone(),
                    event: None,
                    error: None,
                },
            },
            Err(error) => TemplatePreviewResponse {
                matched: false,
                template_name: template.name.clone(),
                event: None,
                error: Some(error),
            },
        }
    }

    pub fn detect_template(&self, sample_lines: &[String]) -> Vec<TemplateMatchSummary> {
        let mut summaries: Vec<TemplateMatchSummary> = self
            .templates
            .iter()
            .map(|template| {
                let matched_lines = sample_lines
                    .iter()
                    .filter(|line| template.pattern.is_match(line))
                    .count();
                let total_lines = sample_lines.len();
                let success_rate = if total_lines == 0 {
                    0.0
                } else {
                    matched_lines as f32 / total_lines as f32
                };

                TemplateMatchSummary {
                    template_name: template.name.clone(),
                    matched_lines,
                    total_lines,
                    success_rate,
                }
            })
            .collect();

        summaries.sort_by(|left, right| {
            right
                .success_rate
                .partial_cmp(&left.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(right.matched_lines.cmp(&left.matched_lines))
        });

        summaries
    }

    pub fn parse_line_to_event(&self, line: &str) -> Option<TemplateParseResult> {
        for template in &self.templates {
            if let Some(event) = Self::parse_with_template(line, template) {
                return Some(TemplateParseResult {
                    template_name: template.name.clone(),
                    event,
                });
            }
        }

        None
    }

    pub fn parse_line(&self, line: &str, line_number: u64, offset: u64) -> LogEntry {
        match self.parse_line_to_event(line) {
            Some(result) => {
                let timestamp = result
                    .event
                    .timestamp
                    .as_deref()
                    .and_then(Self::parse_timestamp)
                    .unwrap_or(0);
                let level = result
                    .event
                    .level
                    .as_deref()
                    .map(LogLevel::from_str)
                    .unwrap_or(LogLevel::Other);
                let source = result.event.source.as_deref().unwrap_or("Unknown");

                LogEntry::new(
                    line_number,
                    self.file_id,
                    line_number,
                    timestamp,
                    level,
                    source,
                    &result.event.message,
                    offset,
                    line.len() as u32,
                    &result.template_name,
                    result.event.extra,
                )
            }
            None => LogEntry::new(
                line_number,
                self.file_id,
                line_number,
                0,
                LogLevel::Other,
                "Unknown",
                if line.len() > 200 { &line[..200] } else { line },
                offset,
                line.len() as u32,
                &self.fallback_template_name,
                HashMap::new(),
            ),
        }
    }

    pub fn parse_lines_parallel(&self, lines: &[(&str, u64, u64)]) -> Vec<LogEntry> {
        use rayon::prelude::*;

        lines
            .par_iter()
            .map(|(line, line_number, offset)| self.parse_line(line, *line_number, *offset))
            .collect()
    }

    fn parse_with_template(line: &str, template: &LogTemplate) -> Option<LogEvent> {
        let captures = template.pattern.captures(line)?;
        let mut extra = HashMap::new();

        let resolve = |standard_field: &str| -> Option<String> {
            let capture_name = template.field_mapping.get(standard_field)?;
            captures
                .name(capture_name)
                .map(|value| value.as_str().trim().to_string())
                .filter(|value| !value.is_empty())
        };

        let timestamp = resolve("timestamp");
        let level = resolve("level");
        let source = resolve("source");
        let message = resolve("message").unwrap_or_else(|| line.to_string());

        for capture_name in template.pattern.capture_names().flatten() {
            let mapped = template
                .field_mapping
                .values()
                .any(|value| value == capture_name);
            if mapped {
                continue;
            }

            if let Some(value) = captures.name(capture_name) {
                extra.insert(capture_name.to_string(), value.as_str().trim().to_string());
            }
        }

        Some(LogEvent {
            timestamp,
            level,
            source,
            message,
            extra,
        })
    }

    pub fn parse_timestamp(value: &str) -> Option<i64> {
        let normalized = value.replace(',', ".");
        let naive_formats = [
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%m/%d/%Y %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
        ];

        for format in naive_formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, format) {
                return Some(dt.and_utc().timestamp_millis());
            }
        }

        if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
            return Some(dt.with_timezone(&Utc).timestamp_millis());
        }

        None
    }
}
