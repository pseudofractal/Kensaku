use console::{Style, measure_text_width, Color};
use crate::config::FractalType;
use crate::config::Config;
use crate::fractal;
use crate::info::{cpu, memory, uptime, system, user, disk, ip, packages, shell, wm};

pub fn print_info(config: &Config) {
    let accent_color = config.accent_color.unwrap_or(Color::White);
    let mut ascii_lines: Vec<String> = Vec::new();

    let mut ascii_width = 60;
    if let Some(art_cfg) = &config.art {
        ascii_width = art_cfg.max_length.unwrap_or(60);
        match art_cfg.fractal {
            Some(FractalType::None) | None => {}
            _ => {
                let width = art_cfg.max_length.unwrap_or(60);
                let height = art_cfg.max_breadth.unwrap_or(20);
                let fractal_type = art_cfg.fractal.clone().unwrap_or(FractalType::MandelbrotSet);
                let style = Style::new().fg(accent_color);
                ascii_lines = fractal::generate_ascii(fractal_type, width, height)
                        .into_iter()
                        .map(|line| style.apply_to(line).to_string())
                        .collect();
            }
        }
    }
    
    let mut info_lines: Vec<String> = Vec::new();

    if config.user_host.unwrap_or(false) {
        if let (Some(user), Some(host)) = (user::username(), system::hostname()) {
            info_lines.push(format_line("", "User", &format!("{}@{}", user, host), accent_color));
        }
    }

    if config.os.unwrap_or(false) {
        if let Some(name) = system::os_pretty_name() {
            info_lines.push(format_line("", "OS", &format!("{} {}", &name, &system::arch()), accent_color));
        }
    }

    if config.kernel.unwrap_or(false) {
        if let Some(k) = system::kernel_version() {
            info_lines.push(format_line("󰌽", "Kernel", &k, accent_color));
        }
    }

    if config.shell.unwrap_or(false) {
        info_lines.push(format_line("", "Shell", &shell::get_shell(), accent_color));
    }

    if config.cpu.unwrap_or(false) {
        let cpu_model = cpu::get_cpu_model().unwrap_or_else(|| "Unknown CPU".to_string());
        info_lines.push(format_line("", "CPU", &cpu_model, accent_color));
    }

    if config.memory.unwrap_or(false) {
        let (used, total) = memory::get_memory_usage().unwrap_or((0, 0));
        info_lines.push(format_line("󰽙", "Memory", &format!("{} MiB / {} MiB", used, total), accent_color));
    }

    if config.uptime.unwrap_or(false) {
        let up = uptime::get_uptime().unwrap_or_else(|| "Unknown".to_string());
        info_lines.push(format_line("󱑋", "Uptime", &up, accent_color));
    }

    if config.disk.unwrap_or(false) {
        if let Some((used, total)) = disk::get_disk_usage() {
            info_lines.push(format_line("", "Disk", &format!("{:.3} GiB / {:.3} TiB", used, total), accent_color));
        }
    }

    if config.ip.unwrap_or(false) {
        if let Some(ip_addr) = ip::get_ip_address() {
            info_lines.push(format_line("", "Ip", &ip_addr, accent_color));
        }
    }

    if config.packages.unwrap_or(false) {
        if let Some(pack) = packages::get_package_count() {
            info_lines.push(format_line("󰂾", "Packages", &pack, accent_color));
        }
    }

    if config.wm.unwrap_or(false) {
        info_lines.push(format_line("󰖯", "WM", &wm::get_window_manager(), accent_color));
    }

    let content_width = info_lines.iter().map(|s| measure_text_width(s)).max().unwrap_or(0);
    let mut bordered_info_lines = Vec::new();
    bordered_info_lines.push(format!("╭{}╮", "─".repeat(content_width + 2)));
    for line in info_lines.iter() {
        let visible_len = measure_text_width(line);
        let pad = content_width.saturating_sub(visible_len);
        let padded_line = format!("{}{}", line, " ".repeat(pad));
        bordered_info_lines.push(format!("│ {} │", padded_line));
    }
    bordered_info_lines.push(format!("╰{}╯", "─".repeat(content_width + 2)));

    let max_len = ascii_lines.len().max(bordered_info_lines.len());
    let mut padded_ascii = ascii_lines;
    let mut padded_info = bordered_info_lines;

    padded_ascii.resize(max_len, "".to_string());
    padded_info.resize(max_len, "".to_string());

    for (a, i) in padded_ascii.iter().zip(padded_info.iter()) {
        println!("{:<width$}  {}", a, i, width = ascii_width);
    }
}

fn format_line(icon: &str, label: &str, value: &str, color: Color) -> String {
    let style = Style::new().fg(color);
    format!("{}  {}: {}", style.apply_to(icon), style.apply_to(label), value)
}
