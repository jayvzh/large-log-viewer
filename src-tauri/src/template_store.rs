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
            println!("DEBUG: current_exe = {:?}", exe_path);
            if let Some(exe_dir) = exe_path.parent() {
                println!("DEBUG: exe_dir = {:?}", exe_dir);
                // 检查是否是开发模式（通过 CARGO_MANIFEST_DIR 环境变量或临时目录判断）
                let is_dev = std::env::var("CARGO_MANIFEST_DIR").is_ok() || 
                             exe_dir.to_string_lossy().contains("AppData\\Local\\Temp") ||
                             exe_dir.to_string_lossy().contains("target");
                
                if is_dev {
                    // 开发模式：使用 sample/data 目录
                    let project_root = std::env::var("CARGO_MANIFEST_DIR")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| PathBuf::from("e:\\Code\\github\\large-log-viewer\\src-tauri"));
                    let path = project_root.parent().unwrap().join("sample").join("data").join("templates.json");
                    println!("DEBUG: dev mode, using path = {:?}", path);
                    return path;
                } else {
                    // 生产模式：使用可执行文件所在目录的 data 目录
                    let path = exe_dir.join("data").join("templates.json");
                    println!("DEBUG: prod mode, using path = {:?}", path);
                    return path;
                }
            }
        }
        
        // 默认使用当前目录的 data 目录
        let path = PathBuf::from("data").join("templates.json");
        println!("DEBUG: default, using path = {:?}", path);
        path
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
        let builtin = LogTemplateParser::get_builtin_templates();
        
        // 过滤掉用户模板中可能存在的内置模板（通过名称去重）
        let builtin_names: std::collections::HashSet<&String> = builtin.iter().map(|t| &t.name).collect();
        templates.retain(|t| !builtin_names.contains(&t.name));
        
        // 合并用户模板和内置模板
        let mut result = templates;
        result.extend(builtin);
        
        println!("DEBUG: get_all_templates returning {} templates ({} user + {} builtin)", 
                 result.len(), 
                 result.iter().filter(|t| !t.is_builtin).count(),
                 result.iter().filter(|t| t.is_builtin).count());
        
        Ok(result)
    }
    
    pub async fn get_template(&self, name: &str) -> Result<Option<LogTemplate>, String> {
        let templates = self.load_templates().await?;
        Ok(templates.into_iter().find(|t| t.name == name))
    }
}
