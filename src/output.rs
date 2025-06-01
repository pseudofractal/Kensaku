use crate::config::Config;
use crate::info::{cpu, memory, uptime, system, user, disk, ip};

pub fn print_info(config: &Config) {
    let ascii_lines: Vec<String> = config
        .art
        .as_ref()
        .map(|art| art.lines().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let mut info_lines: Vec<String> = Vec::new();

    if config.user_host.unwrap_or(false) {
        if let (Some(user), Some(host)) = (user::username(), system::hostname()) {
            info_lines.push(format_line("", "User", &format!("{}@{}", user, host)));
        }
    }

    if config.os.unwrap_or(false) {
        if let Some(name) = system::os_pretty_name() {
            info_lines.push(format_line("", "OS", &format!("{} {}", &name, &system::arch())));
        }
    }

    if config.kernel.unwrap_or(false) {
        if let Some(k) = system::kernel_version() {
            info_lines.push(format_line("󰌽", "Kernel", &k));
        }
    }

    if config.cpu.unwrap_or(false) {
        let cpu_model = cpu::get_cpu_model().unwrap_or_else(|| "Unknown CPU".to_string());
        info_lines.push(format_line("", "CPU", &cpu_model));
    }

    if config.memory.unwrap_or(false) {
        let (used, total) = memory::get_memory_usage().unwrap_or((0, 0));
        info_lines.push(format_line("󰽙", "Memory", &format!("{} MiB / {} MiB", used, total)));
    }

    if config.uptime.unwrap_or(false) {
        let up = uptime::get_uptime().unwrap_or_else(|| "Unknown".to_string());
        info_lines.push(format_line("󱑋", "Uptime", &up));
    }

    if config.disk.unwrap_or(false) {
        if let Some((used, total)) = disk::get_disk_usage() {
            info_lines.push(format_line("", "Disk", &format!("{:.3} GiB / {:.3} TiB", used, total)));
        }
    }

    if config.ip.unwrap_or(false) {
        if let Some(ip) = ip::get_ip_address() {
            info_lines.push(format_line("", "Ip", &ip));
        }
    }

    let content_width = info_lines.iter().map(|s| s.len()).max().unwrap_or(0);

    let mut bordered_info_lines = Vec::new();
    bordered_info_lines.push(format!("╭{}╮", "─".repeat(content_width + 2)));
    for line in info_lines.iter() {
        bordered_info_lines.push(format!("│ {:width$} │", line, width = content_width));
    }
    bordered_info_lines.push(format!("╰{}╯", "─".repeat(content_width + 2)));

    let max_len = ascii_lines.len().max(bordered_info_lines.len());
    let mut padded_ascii = ascii_lines;
    let mut padded_info = bordered_info_lines;

    padded_ascii.resize(max_len, "".to_string());
    padded_info.resize(max_len, "".to_string());

    for (ascii, info) in padded_ascii.iter().zip(padded_info.iter()) {
        println!("{:<30}  {}", ascii, info);
    }
}

fn format_line(icon: &str, label: &str, value: &str) -> String {
    format!("{}  {}: {}", icon, label, value)
}

