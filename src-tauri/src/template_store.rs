use crate::models::LogTemplate;
use crate::parser::LogTemplateParser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fs;

pub struct TemplateStore {
    templates_path: PathBuf,
    lock: Arc<RwLock<()>>,
}

impl TemplateStore {
    pub fn new() -> Self {
        let templates_path = Self::resolve_templates_path();
        
        Self {
            templates_path,
            lock: Arc::new(RwLock::new(())),
        }
    }
    
    fn resolve_templates_path() -> PathBuf {
        // 在开发模式下，使用 sample/data 目录作为模板存储路径
        // 在生产模式下，使用可执行文件所在目录的 data 目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // 检查是否是开发模式（临时目录）
                if exe_dir.to_string_lossy().contains("AppData\\Local\\Temp") {
                    // 开发模式：使用 sample/data 目录
                    let project_root = PathBuf::from("e:\\Code\\github\\large-log-viewer");
                    return project_root.join("sample").join("data").join("templates.json");
                } else {
                    // 生产模式：使用可执行文件所在目录的 data 目录
                    return exe_dir.join("data").join("templates.json");
                }
            }
        }
        
        // 默认使用当前目录的 data 目录
        PathBuf::from("data").join("templates.json")
    }
    
    pub fn get_templates_path(&self) -> &PathBuf {
        &self.templates_path
    }
    
    pub async fn load_templates(&self) -> Result<Vec<LogTemplate>, String> {
        let _guard = self.lock.read().await;
        self.load_templates_internal()
    }
    
    pub async fn save_templates(&self, templates: &[LogTemplate]) -> Result<(), String> {
        let _guard = self.lock.write().await;
        self.save_templates_internal(templates)
    }
    
    pub async fn add_template(&self, template: &LogTemplate) -> Result<(), String> {
        let _guard = self.lock.write().await;
        
        let mut templates = self.load_templates_internal()?;
        
        if let Some(pos) = templates.iter().position(|t| t.name == template.name) {
            templates[pos] = template.clone();
        } else {
            templates.push(template.clone());
        }
        
        self.save_templates_internal(&templates)?;
        
        Ok(())
    }
    
    pub async fn remove_template(&self, name: &str) -> Result<(), String> {
        let _guard = self.lock.write().await;
        
        let mut templates = self.load_templates_internal()?;
        
        templates.retain(|t| t.name != name);
        
        self.save_templates_internal(&templates)?;
        
        Ok(())
    }
    
    fn load_templates_internal(&self) -> Result<Vec<LogTemplate>, String> {
        if !self.templates_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.templates_path)
            .map_err(|e| format!("Failed to read templates file: {}", e))?;
        
        let templates: Vec<LogTemplate> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse templates: {}", e))?;
        
        Ok(templates)
    }
    
    fn save_templates_internal(&self, templates: &[LogTemplate]) -> Result<(), String> {
        if let Some(parent) = self.templates_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create templates directory: {}", e))?;
        }
        
        let content = serde_json::to_string_pretty(templates)
            .map_err(|e| format!("Failed to serialize templates: {}", e))?;
        
        fs::write(&self.templates_path, content)
            .map_err(|e| format!("Failed to write templates file: {}", e))?;
        
        Ok(())
    }
    
    pub async fn get_all_templates(&self) -> Result<Vec<LogTemplate>, String> {
        let mut templates = self.load_templates().await?;
        let mut builtin = LogTemplateParser::get_builtin_templates();
        templates.append(&mut builtin);
        Ok(templates)
    }
    
    pub async fn get_template(&self, name: &str) -> Result<Option<LogTemplate>, String> {
        let templates = self.load_templates().await?;
        Ok(templates.into_iter().find(|t| t.name == name))
    }
}
