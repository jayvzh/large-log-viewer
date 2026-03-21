use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100000);
    let output = args.get(2).cloned().unwrap_or_else(|| "test.log".to_string());
    
    let file = File::create(&output)?;
    let mut writer = BufWriter::new(file);
    
    let levels = ["FATAL", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
    let loggers = [
        "com.example.service.UserService",
        "com.example.dao.UserDao",
        "com.example.controller.UserController",
        "com.example.util.DateUtils",
        "com.example.config.AppConfig",
        "com.example.processor.TaskProcessor",
        "com.example.handler.RequestHandler",
        "com.example.cache.RedisClient",
    ];
    
    let messages = [
        "用户登录成功 userId={}",
        "数据库查询完成 rows={}",
        "请求处理开始 requestId={}",
        "缓存命中 key={}",
        "配置加载完成 env={}",
        "任务执行完成 taskId={}",
        "连接建立成功 host={}",
        "数据同步完成 count={}",
    ];
    
    println!("Generating {} log entries...", count);
    
    for i in 0..count {
        let level = levels[i % levels.len()];
        let logger = loggers[i % loggers.len()];
        let message = messages[i % messages.len()];
        
        let hour = (i / 3600) % 24;
        let minute = (i / 60) % 60;
        let second = i % 60;
        let millis = (i * 123) % 1000;
        
        writeln!(
            writer,
            "2024-01-15 {:02}:{:02}:{:02}.{:03} {}  {} - {}",
            hour + 8,  // UTC+8
            minute,
            second,
            millis,
            level,
            logger,
            message.replace("{}", &format!("{}", i % 10000))
        )?;
    }
    
    writer.flush()?;
    
    let metadata = std::fs::metadata(&output)?;
    println!("Generated {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);
    println!("Output file: {}", output);
    
    Ok(())
}
