use std::fs;

pub fn get_cpu_model() -> Option<String> {
    let contents = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in contents.lines() {
        if line.starts_with("model name") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}

