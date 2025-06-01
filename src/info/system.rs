use std::fs;

pub fn hostname() -> Option<String> {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn kernel_version() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn os_pretty_name() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Some(line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string());
        }
    }
    None
}

pub fn arch() -> String {
    std::env::consts::ARCH.to_string()
}

