use regex::Regex;

fn main() {
    let pattern = r"^\[(?P<timestamp>[^\]]+)\] \[(?P<level>[^\]]+)\] \[(?P<source>[^\]]+)\] \[(?P<thread_id>[^\]]+)\] (?P<message>.+?) - ID:(?P<id>\d+)$";
    
    match Regex::new(pattern) {
        Ok(re) => {
            println!("Regex compiled successfully!");
            
            let test_line = "[2026-03-21 16:03:33.999] [FATAL] [Worker] [T14] Received shutdown signal - ID:5283";
            
            if let Some(caps) = re.captures(test_line) {
                println!("MATCHED!");
                for name in re.capture_names().flatten() {
                    if let Some(value) = caps.name(name) {
                        println!("  {}: {}", name, value.as_str());
                    }
                }
            } else {
                println!("NO MATCH!");
            }
        }
        Err(e) => {
            println!("Regex error: {}", e);
        }
    }
}
