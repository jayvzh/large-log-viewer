use crate::models::*;
use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 将 style 字符串转换为 CSS 类名
fn style_to_css_class(style: &str) -> String {
    let mut classes = Vec::new();
    let style_lower = style.to_lowercase();
    
    // 检查是否是 CSS 语法（包含冒号）
    if style_lower.contains(':') {
        for part in style_lower.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            
            if let Some((property, value)) = part.split_once(':') {
                let property = property.trim();
                let value = value.trim();
                
                match property {
                    "color" => {
                        if value.starts_with('#') {
                            let color = value.trim_start_matches('#');
                            classes.push(format!("hl-{}", color));
                        } else {
                             match value {
                                "red" => classes.push("hl-red".to_string()),
                                "green" => classes.push("hl-green".to_string()),
                                "blue" => classes.push("hl-blue".to_string()),
                                "yellow" => classes.push("hl-yellow".to_string()),
                                "cyan" => classes.push("hl-cyan".to_string()),
                                "purple" => classes.push("hl-purple".to_string()),
                                "gray" | "grey" => classes.push("hl-gray".to_string()),
                                _ => {
                                    // 对于未知的颜色值，尝试直接使用它作为类名的一部分
                                    classes.push(format!("hl-{}", value));
                                }
                            }
                        }
                    }
                    "font-weight" if value == "bold" => {
                        classes.push("hl-bold".to_string());
                    }
                    "font-style" if value == "italic" => {
                        classes.push("hl-italic".to_string());
                    }
                    "text-decoration" if value == "underline" => {
                        classes.push("hl-underline".to_string());
                    }
                    _ => {} // 忽略其他属性
                }
            }
        }
    } else {
        // 处理空格分隔的样式
        for part in style_lower.split_whitespace() {
            if part.starts_with('#') {
                let color = part.trim_start_matches('#');
                classes.push(format!("hl-{}", color));
            } else {
                match part {
                    "red" => classes.push("hl-red".to_string()),
                    "green" => classes.push("hl-green".to_string()),
                    "blue" => classes.push("hl-blue".to_string()),
                    "yellow" => classes.push("hl-yellow".to_string()),
                    "cyan" => classes.push("hl-cyan".to_string()),
                    "purple" => classes.push("hl-purple".to_string()),
                    "gray" | "grey" => classes.push("hl-gray".to_string()),
                    "bold" => classes.push("hl-bold".to_string()),
                    "italic" => classes.push("hl-italic".to_string()),
                    "underline" => classes.push("hl-underline".to_string()),
                    _ => {
                        // 对于未知的样式值，尝试直接使用它作为类名的一部分
                        classes.push(format!("hl-{}", part));
                    }
                }
            }
        }
    }
    
    classes.join(" ")
}

pub struct HighlightEngine {
    profiles: Vec<HighlightProfile>,
    cached_engines: RwLock<HashMap<String, Arc<CompiledProfile>>>,
}

struct CompiledProfile {
    field_rules: Vec<(String, HashMap<String, String>, Option<String>)>,
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
                HighlightRule::Field { field, style_map, default_style } => {
                    let mut a: HashMap<String, String> = HashMap::new();
                    for (k, v) in style_map {
                        a.insert(k.to_lowercase(), v.clone());
                    }
                    field_rules.push((field.clone(), a, default_style.clone()));
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

        // 1. Field rules (highest priority)
        self.apply_field_rules(entry, &compiled.field_rules, &mut spans);
        
        // 2. Keyword, Token, Regex rules on message
        let message = entry.message_str();
        let offset = 0;
        self.apply_keyword_rules(message, offset, compiled, &mut spans);
        self.apply_token_rules(message, offset, &compiled.token_regexes, &mut spans);
        self.apply_regex_rules(message, offset, &compiled.regex_regexes, &mut spans);

        // 3. Keyword, Token, Regex rules on extra fields
        for (field, value) in &entry.extra {
            self.apply_rules_to_extra_field(field, value, compiled, &mut spans);
        }

        // Merge overlapping spans
        self.merge_spans(&mut spans);

        spans
    }

    fn apply_field_rules(&self, entry: &LogEntry, rules: &[(String, HashMap<String, String>, Option<String>)], spans: &mut Vec<HighlightSpan>) {
        for (field, style_map, default_style) in rules {
            let (value, offset): (&str, usize) = match field.as_str() {
                "level" => (entry.level.as_str(), 0),
                "source" => (entry.source_str(), 0),
                "message" => (entry.message_str(), 0),
                _ => {
                    if let Some(val) = entry.extra.get(field) {
                        (val.as_str(), 0)
                    } else {
                        ("", 0)
                    }
                }
            };

            if value.is_empty() {
                continue;
            }

            let style = style_map.get(&value.to_lowercase())
                .or_else(|| default_style.as_ref());
            
            if let Some(style) = style {
                let css_class = style_to_css_class(style);
                if field == "level" || field == "source" || field == "message" {
                    spans.push(HighlightSpan {
                        start: offset,
                        end: offset + value.len(),
                        class: css_class,
                    });
                } else {
                    let span_class = format!("extra-field-{}-{}", field, css_class);
                    if !spans.iter().any(|s| s.class == span_class) {
                        spans.push(HighlightSpan { start: 0, end: 0, class: span_class });
                    }
                }
            }
        }
    }

    fn apply_keyword_rules(&self, text: &str, offset: usize, compiled: &CompiledProfile, spans: &mut Vec<HighlightSpan>) {
        if let Some(automaton) = &compiled.keyword_automaton {
            for mat in automaton.find_iter(text) {
                let word = &text[mat.start()..mat.end()];
                if let Some(style) = compiled.keyword_styles.get(word) {
                    let css_class = style_to_css_class(style);
                    spans.push(HighlightSpan {
                        start: offset + mat.start(),
                        end: offset + mat.end(),
                        class: css_class,
                    });
                }
            }
        }
    }

    fn apply_token_rules(&self, text: &str, offset: usize, regexes: &[(Regex, String)], spans: &mut Vec<HighlightSpan>) {
        for (regex, style) in regexes {
            for mat in regex.find_iter(text) {
                let css_class = style_to_css_class(style);
                spans.push(HighlightSpan {
                    start: offset + mat.start(),
                    end: offset + mat.end(),
                    class: css_class,
                });
            }
        }
    }

    fn apply_regex_rules(&self, text: &str, offset: usize, regexes: &[(Regex, String)], spans: &mut Vec<HighlightSpan>) {
        for (regex, style) in regexes {
            for mat in regex.find_iter(text) {
                let css_class = style_to_css_class(style);
                spans.push(HighlightSpan {
                    start: offset + mat.start(),
                    end: offset + mat.end(),
                    class: css_class,
                });
            }
        }
    }

    fn apply_rules_to_extra_field(&self, field: &str, value: &str, compiled: &CompiledProfile, spans: &mut Vec<HighlightSpan>) {
        // Apply regex rules to find all matches within the extra field value
        // This allows highlighting specific parts of the field (like HTTP methods in request)
        for (regex, style) in &compiled.regex_regexes {
            for _mat in regex.find_iter(value) {
                let css_class = style_to_css_class(style);
                let span_class = format!("extra-field-{}-{}", field, css_class);
                
                if !spans.iter().any(|s| s.class == span_class) {
                    spans.push(HighlightSpan { 
                        start: 0, 
                        end: 0, 
                        class: span_class,
                    });
                }
            }
        }
        
        // Also check keyword and token rules
        let mut style = None;

        // Keyword
        if let Some(automaton) = &compiled.keyword_automaton {
            if let Some(mat) = automaton.find_iter(value).next() {
                let word = &value[mat.start()..mat.end()];
                if let Some(s) = compiled.keyword_styles.get(word) {
                    style = Some(s.clone());
                }
            }
        }
        
        // Token
        if style.is_none() {
            for (regex, s) in &compiled.token_regexes {
                if regex.is_match(value) {
                    style = Some(s.clone());
                    break;
                }
            }
        }

        if let Some(s) = style {
            let css_class = style_to_css_class(&s);
            let span_class = format!("extra-field-{}-{}", field, css_class);
            if !spans.iter().any(|s| s.class == span_class) {
                spans.push(HighlightSpan { start: 0, end: 0, class: span_class });
            }
        }
    }

    fn merge_spans(&self, spans: &mut Vec<HighlightSpan>) {
        if spans.is_empty() {
            return;
        }

        // Separate extra field spans
        let mut extra_spans = Vec::new();
        let mut content_spans = Vec::new();
        for span in spans.drain(..) {
            if span.start == 0 && span.end == 0 {
                extra_spans.push(span);
            } else {
                content_spans.push(span);
            }
        }

        if content_spans.is_empty() {
            *spans = extra_spans;
            return;
        }

        // Sort content spans by start position
        content_spans.sort_by(|a, b| a.start.cmp(&b.start));

        let mut merged = Vec::new();
        let mut current = content_spans[0].clone();

        for span in content_spans.iter().skip(1) {
            if span.start < current.end {
                // Overlapping, merge classes. Higher priority (current) styles are preserved.
                let mut current_classes: Vec<_> = current.class.split_whitespace().collect();
                let new_classes: Vec<_> = span.class.split_whitespace().collect();
                for new_class in new_classes {
                    if !current_classes.contains(&new_class) {
                        current_classes.push(new_class);
                    }
                }
                current.class = current_classes.join(" ");
                current.end = current.end.max(span.end);
            } else {
                merged.push(current);
                current = span.clone();
            }
        }
        merged.push(current);

        *spans = merged;
        spans.append(&mut extra_spans);
    }
}
