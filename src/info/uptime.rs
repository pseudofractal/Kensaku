use std::fs;

pub fn get_uptime() -> Option<String> {
    let contents = fs::read_to_string("/proc/uptime").ok()?;
    let seconds: f64 = contents.split_whitespace().next()?.parse().ok()?;
    let secs = seconds as u64;

    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;

    Some(format!("{}h {}m", hours, mins))
}

