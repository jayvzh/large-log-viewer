use crate::models::*;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct HighlightEngine {
    profiles: Vec<HighlightProfile>,
    cached_engines: RwLock<HashMap<String, Arc<CompiledProfile>>>,
}

struct CompiledProfile {
    field_rules: Vec<(String, HashMap<String, String>)>,
    keyword_automaton: Option<AhoCorasick>,
    keyword_styles: HashMap<String, String>,
    token_regexes: Vec<(Regex, String)>,
    regex_regexes: Vec<(Regex, String)>,
}

impl HighlightEngine {
    pub fn new(profiles: Vec<HighlightProfile>) -> Self {
        Self {
            profiles,
            cached_engines: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_profiles(&self) -> &Vec<HighlightProfile> {
        &self.profiles
    }

    pub fn update_profiles(&mut self, profiles: Vec<HighlightProfile>) {
        self.profiles = profiles;
        let mut cached_engines = self.cached_engines.write().unwrap();
        cached_engines.clear();
    }

    pub fn process_entry(&self, entry: &LogEntry, profile_name: &str) -> Vec<HighlightSpan> {
        let compiled = self.compile_profile(profile_name);
        if let Some(compiled) = compiled {
            self.process_with_compiled(entry, &compiled)
        } else {
            Vec::new()
        }
    }

    fn compile_profile(&self, profile_name: &str) -> Option<Arc<CompiledProfile>> {
        {
            let cached_engines = self.cached_engines.read().unwrap();
            if let Some(cached) = cached_engines.get(profile_name) {
                return Some(cached.clone());
            }
        }

        let profile = self.profiles.iter().find(|p| p.name == profile_name)?;

        let mut field_rules = Vec::new();
        let mut keyword_patterns = Vec::new();
        let mut keyword_styles = HashMap::new();
        let mut token_regexes = Vec::new();
        let mut regex_regexes = Vec::new();

        for rule in &profile.rules {
            match rule {
                HighlightRule::Field { field, style_map } => {
                    field_rules.push((field.clone(), style_map.clone()));
                }
                HighlightRule::Keyword { words, style } => {
                    for word in words {
                        keyword_patterns.push(word);
                        keyword_styles.insert(word.clone(), style.clone());
                    }
                }
                HighlightRule::Token { pattern, style } => {
                    if let Ok(regex) = Regex::new(pattern) {
                        token_regexes.push((regex, style.clone()));
                    }
                }
                HighlightRule::Regex { pattern, style } => {
                    if let Ok(regex) = Regex::new(pattern) {
                        regex_regexes.push((regex, style.clone()));
                    }
                }
            }
        }

        let keyword_automaton = if !keyword_patterns.is_empty() {
            AhoCorasick::new(&keyword_patterns).ok()
        } else {
            None
        };

        let compiled = Arc::new(CompiledProfile {
            field_rules,
            keyword_automaton,
            keyword_styles,
            token_regexes,
            regex_regexes,
        });

        let mut cached_engines = self.cached_engines.write().unwrap();
        cached_engines.insert(profile_name.to_string(), compiled.clone());
        Some(compiled)
    }

    fn process_with_compiled(&self, entry: &LogEntry, compiled: &CompiledProfile) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        let raw_content = format!("{} {} {}", entry.source_str(), entry.message_str(), 
            entry.extra.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" "));

        // 1. Field rules (highest priority)
        self.process_field_rules(entry, &compiled.field_rules, &mut spans);

        // 2. Keyword rules
        self.process_keyword_rules(&raw_content, compiled, &mut spans);

        // 3. Token rules
        self.process_token_rules(&raw_content, &compiled.token_regexes, &mut spans);

        // 4. Regex rules (lowest priority)
        self.process_regex_rules(&raw_content, &compiled.regex_regexes, &mut spans);

        // Merge overlapping spans
        self.merge_spans(&mut spans);

        spans
    }

    fn process_field_rules(&self, entry: &LogEntry, rules: &[(String, HashMap<String, String>)], spans: &mut Vec<HighlightSpan>) {
        for (field, style_map) in rules {
            match field.as_str() {
                "level" => {
                    let level_str = entry.level.as_str();
                    if let Some(style) = style_map.get(level_str) {
                        // Find level in the raw content
                        let raw_content = format!("{} {} {}", entry.source_str(), entry.message_str(), 
                            entry.extra.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" "));
                        if let Some(start) = raw_content.find(level_str) {
                            let end = start + level_str.len();
                            spans.push(HighlightSpan {
                                start,
                                end,
                                class: style.clone(),
                            });
                        }
                    }
                }
                "source" => {
                    let source_str = entry.source_str();
                    if !source_str.is_empty() && source_str != "Unknown" {
                        if let Some(style) = style_map.get(source_str) {
                            if let Some(start) = source_str.find(source_str) {
                                let end = start + source_str.len();
                                spans.push(HighlightSpan {
                                    start,
                                    end,
                                    class: style.clone(),
                                });
                            }
                        }
                    }
                }
                "message" => {
                    let message_str = entry.message_str();
                    if let Some(style) = style_map.get(message_str) {
                        let source_len = entry.source_str().len() + 1; // +1 for space
                        spans.push(HighlightSpan {
                            start: source_len,
                            end: source_len + message_str.len(),
                            class: style.clone(),
                        });
                    }
                }
                _ => {
                    // Extra fields
                    if let Some(value) = entry.extra.get(field) {
                        if let Some(style) = style_map.get(value) {
                            let raw_content = format!("{} {} {}", entry.source_str(), entry.message_str(), 
                                entry.extra.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(" "));
                            let pattern = format!("{}={}", field, value);
                            if let Some(start) = raw_content.find(&pattern) {
                                let end = start + pattern.len();
                                spans.push(HighlightSpan {
                                    start,
                                    end,
                                    class: style.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_keyword_rules(&self, content: &str, compiled: &CompiledProfile, spans: &mut Vec<HighlightSpan>) {
        if let Some(automaton) = &compiled.keyword_automaton {
            for mat in automaton.find_iter(content) {
                let word = &content[mat.start()..mat.end()];
                if let Some(style) = compiled.keyword_styles.get(word) {
                    spans.push(HighlightSpan {
                        start: mat.start(),
                        end: mat.end(),
                        class: style.clone(),
                    });
                }
            }
        }
    }

    fn process_token_rules(&self, content: &str, regexes: &[(Regex, String)], spans: &mut Vec<HighlightSpan>) {
        for (regex, style) in regexes {
            for mat in regex.find_iter(content) {
                spans.push(HighlightSpan {
                    start: mat.start(),
                    end: mat.end(),
                    class: style.clone(),
                });
            }
        }
    }

    fn process_regex_rules(&self, content: &str, regexes: &[(Regex, String)], spans: &mut Vec<HighlightSpan>) {
        for (regex, style) in regexes {
            for mat in regex.find_iter(content) {
                spans.push(HighlightSpan {
                    start: mat.start(),
                    end: mat.end(),
                    class: style.clone(),
                });
            }
        }
    }

    fn merge_spans(&self, spans: &mut Vec<HighlightSpan>) {
        if spans.is_empty() {
            return;
        }

        // Sort spans by start position
        spans.sort_by(|a, b| a.start.cmp(&b.start));

        let mut merged = Vec::new();
        let mut current = spans[0].clone();

        for span in spans.iter().skip(1) {
            if span.start <= current.end {
                // Overlapping or adjacent, merge
                current.end = current.end.max(span.end);
                // Use the higher priority style (earlier rules have higher priority)
                // Since we process rules in priority order, current span has higher priority
            } else {
                merged.push(current);
                current = span.clone();
            }
        }
        merged.push(current);

        *spans = merged;
    }
}
