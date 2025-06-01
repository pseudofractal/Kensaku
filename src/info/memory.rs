use std::fs;

pub fn get_memory_usage() -> Option<(u64, u64)> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0;
    let mut available = 0;
    let mut free = 0;

    for line in contents.lines() {
        if line.starts_with("MemTotal:") {
            total = extract_kib(line);
        } else if line.starts_with("MemAvailable:") {
            available = extract_kib(line);
        } else if line.starts_with("MemFree:") {
            free = extract_kib(line);
        }
    }

    let used = total.saturating_sub(available.max(free));
    Some((used / 1024, total / 1024)) // Return in MiB
}

fn extract_kib(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

