use crate::models::HighlightProfile;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fs;

pub struct HighlightStore {
    highlights_path: PathBuf,
    lock: Arc<RwLock<()>>,
}

impl HighlightStore {
    pub fn new() -> Self {
        let highlights_path = Self::resolve_highlights_path();
        
        Self {
            highlights_path,
            lock: Arc::new(RwLock::new(())),
        }
    }
    
    fn resolve_highlights_path() -> PathBuf {
        let data_dir = std::env::current_exe()
            .map(|p| p.parent().unwrap_or(&PathBuf::from(".")).join("data"))
            .unwrap_or_else(|_| PathBuf::from("data"));
        
        data_dir.join("highlights.json")
    }
    
    pub fn get_highlights_path(&self) -> &PathBuf {
        &self.highlights_path
    }
    
    pub async fn load_profiles(&self) -> Result<Vec<HighlightProfile>, String> {
        let _guard = self.lock.read().await;
        self.load_profiles_internal()
    }
    
    pub async fn save_profiles(&self, profiles: &[HighlightProfile]) -> Result<(), String> {
        let _guard = self.lock.write().await;
        self.save_profiles_internal(profiles)
    }
    
    pub async fn add_profile(&self, profile: &HighlightProfile) -> Result<(), String> {
        let _guard = self.lock.write().await;
        
        let mut profiles = self.load_profiles_internal()?;
        
        if let Some(pos) = profiles.iter().position(|p| p.name == profile.name) {
            profiles[pos] = profile.clone();
        } else {
            profiles.push(profile.clone());
        }
        
        self.save_profiles_internal(&profiles)?;
        
        Ok(())
    }
    
    pub async fn remove_profile(&self, name: &str) -> Result<(), String> {
        let _guard = self.lock.write().await;
        
        let mut profiles = self.load_profiles_internal()?;
        
        profiles.retain(|p| p.name != name);
        
        self.save_profiles_internal(&profiles)?;
        
        Ok(())
    }
    
    fn load_profiles_internal(&self) -> Result<Vec<HighlightProfile>, String> {
        if !self.highlights_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.highlights_path)
            .map_err(|e| format!("Failed to read highlights file: {}", e))?;
        
        let profiles: Vec<HighlightProfile> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse highlights: {}", e))?;
        
        Ok(profiles)
    }
    
    fn save_profiles_internal(&self, profiles: &[HighlightProfile]) -> Result<(), String> {
        if let Some(parent) = self.highlights_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create highlights directory: {}", e))?;
        }
        
        let content = serde_json::to_string_pretty(profiles)
            .map_err(|e| format!("Failed to serialize highlights: {}", e))?;
        
        fs::write(&self.highlights_path, content)
            .map_err(|e| format!("Failed to write highlights file: {}", e))?;
        
        Ok(())
    }
    
    pub async fn get_all_profiles(&self) -> Result<Vec<HighlightProfile>, String> {
        let mut profiles = self.load_profiles().await?;
        let mut builtin = Self::get_builtin_profiles();
        profiles.append(&mut builtin);
        Ok(profiles)
    }
    
    pub async fn get_profile(&self, name: &str) -> Result<Option<HighlightProfile>, String> {
        let profiles = self.load_profiles().await?;
        Ok(profiles.into_iter().find(|p| p.name == name))
    }
    
    pub fn get_builtin_profiles() -> Vec<HighlightProfile> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        vec![
            HighlightProfile {
                name: "general_default".to_string(),
                rules: vec![
                    crate::models::HighlightRule::Field {
                        field: "level".to_string(),
                        style_map: std::collections::HashMap::from([
                            ("FATAL".to_string(), "red bold".to_string()),
                            ("ERROR".to_string(), "red".to_string()),
                            ("WARN".to_string(), "yellow".to_string()),
                            ("INFO".to_string(), "blue".to_string()),
                            ("DEBUG".to_string(), "gray".to_string()),
                            ("TRACE".to_string(), "gray".to_string()),
                        ]),
                    },
                    crate::models::HighlightRule::Keyword {
                        words: vec!["failed", "error", "exception", "timeout", "refused"].into_iter().map(|s| s.to_string()).collect(),
                        style: "red".to_string(),
                    },
                    crate::models::HighlightRule::Keyword {
                        words: vec!["success", "ok", "completed", "started"].into_iter().map(|s| s.to_string()).collect(),
                        style: "green".to_string(),
                    },
                ],
                is_builtin: true,
                created_at: now,
                updated_at: now,
            },
            HighlightProfile {
                name: "web_default".to_string(),
                rules: vec![
                    crate::models::HighlightRule::Field {
                        field: "level".to_string(),
                        style_map: std::collections::HashMap::from([
                            ("FATAL".to_string(), "red bold".to_string()),
                            ("ERROR".to_string(), "red".to_string()),
                            ("WARN".to_string(), "yellow".to_string()),
                            ("INFO".to_string(), "blue".to_string()),
                        ]),
                    },
                    crate::models::HighlightRule::Field {
                        field: "status".to_string(),
                        style_map: std::collections::HashMap::from([
                            ("200".to_string(), "green".to_string()),
                            ("201".to_string(), "green".to_string()),
                            ("301".to_string(), "cyan".to_string()),
                            ("302".to_string(), "cyan".to_string()),
                            ("400".to_string(), "yellow".to_string()),
                            ("404".to_string(), "yellow".to_string()),
                            ("500".to_string(), "red bold".to_string()),
                            ("502".to_string(), "red bold".to_string()),
                            ("503".to_string(), "red bold".to_string()),
                        ]),
                    },
                    crate::models::HighlightRule::Regex {
                        pattern: r"\b\d{1,3}(\.\d{1,3}){3}\b".to_string(),
                        style: "cyan".to_string(),
                    },
                ],
                is_builtin: true,
                created_at: now,
                updated_at: now,
            },
            HighlightProfile {
                name: "security_default".to_string(),
                rules: vec![
                    crate::models::HighlightRule::Keyword {
                        words: vec!["failed", "invalid", "unauthorized", "denied", "blocked"].into_iter().map(|s| s.to_string()).collect(),
                        style: "red bold".to_string(),
                    },
                    crate::models::HighlightRule::Keyword {
                        words: vec!["login", "authentication", "password", "session"].into_iter().map(|s| s.to_string()).collect(),
                        style: "yellow".to_string(),
                    },
                    crate::models::HighlightRule::Regex {
                        pattern: r"\b\d{1,3}(\.\d{1,3}){3}\b".to_string(),
                        style: "cyan".to_string(),
                    },
                ],
                is_builtin: true,
                created_at: now,
                updated_at: now,
            },
        ]
    }
}
