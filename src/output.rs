use crate::config::{Config, FractalType};
use crate::fractal;
use crate::info::{cpu, disk, extra, ip, memory, packages, shell, system, uptime, user, wm};
use console::{measure_text_width, Color, Style};

pub fn print_info(config: &Config) {
  let accent = config.accent_color.unwrap_or(Color::White);
  let style = Style::new().fg(accent);
  let art = config
    .art
    .as_ref()
    .and_then(|art| {
      let fractal = art.fractal?;
      if matches!(fractal, FractalType::None) {
        return None;
      }
      Some(fractal::generate_ascii(
        fractal,
        art.max_length.unwrap_or(60),
        art.max_breadth.unwrap_or(20),
        art,
      ))
    })
    .unwrap_or_default();
  let art_width = art
    .iter()
    .map(|line| measure_text_width(line))
    .max()
    .unwrap_or(0);
  let mut info = Vec::new();
  let mut add =
    |enabled: Option<bool>, icon: &str, label: &str, collect: &dyn Fn() -> Option<String>| {
      if enabled.unwrap_or(false) {
        let value = collect();
        info.push(format_line(
          icon,
          label,
          value.as_deref().unwrap_or("Unavailable"),
          accent,
        ));
      }
    };
  add(config.user_host, "", "User", &|| {
    user::username()
      .zip(system::hostname())
      .map(|(user, host)| format!("{user}@{host}"))
  });
  add(config.host_model, "󰌢", "Host", &extra::host_model);
  add(config.os, "", "OS", &|| {
    system::os_pretty_name().map(|name| format!("{name} {}", system::arch()))
  });
  add(config.kernel, "󰌽", "Kernel", &system::kernel_version);
  add(config.shell, "", "Shell", &shell::get_shell);
  add(config.terminal, "", "Terminal", &extra::terminal);
  add(config.cpu, "", "CPU", &cpu::get_cpu_model);
  add(config.cpu_threads, "󰍛", "CPU threads", &extra::cpu_threads);
  add(config.load, "󰓅", "Load (1/5/15 min)", &extra::load_average);
  add(config.memory, "󰽙", "Memory", &|| {
    memory::get_memory_usage().map(format_usage)
  });
  add(config.swap, "󰓡", "Swap", &|| {
    memory::get_swap_usage().map(format_usage)
  });
  add(config.disk, "", "Disk /", &|| {
    disk::get_disk_usage().map(format_usage)
  });
  add(config.uptime, "󱑋", "Uptime", &uptime::get_uptime);
  add(config.ip, "", "Local IP", &ip::get_ip_address);
  add(
    config.packages,
    "󰂾",
    "Packages",
    &packages::get_package_count,
  );
  add(config.wm, "󰖯", "Desktop", &wm::get_window_manager);

  let content_width = info
    .iter()
    .map(|line| measure_text_width(line))
    .max()
    .unwrap_or(0);
  let mut bordered = vec![format!("╭{}╮", "─".repeat(content_width + 2))];
  for line in info {
    let padding = content_width.saturating_sub(measure_text_width(&line));
    bordered.push(format!("│ {line}{} │", " ".repeat(padding)));
  }
  bordered.push(format!("╰{}╯", "─".repeat(content_width + 2)));

  for index in 0..art.len().max(bordered.len()) {
    let info_line = bordered.get(index).map(String::as_str).unwrap_or("");
    if art_width == 0 {
      println!("{info_line}");
    } else {
      let art_line = art.get(index).map(String::as_str).unwrap_or("");
      let padding = art_width.saturating_sub(measure_text_width(art_line));
      println!(
        "{}{}  {info_line}",
        style.apply_to(art_line),
        " ".repeat(padding)
      );
    }
  }
}

fn format_usage((used, total): (u64, u64)) -> String {
  format!("{} / {}", format_bytes(used), format_bytes(total))
}

fn format_bytes(bytes: u64) -> String {
  const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
  let mut value = bytes as f64;
  let mut unit = 0;
  while value >= 1024.0 && unit < UNITS.len() - 1 {
    value /= 1024.0;
    unit += 1;
  }
  format!("{value:.2} {}", UNITS[unit])
}

fn format_line(icon: &str, label: &str, value: &str, color: Color) -> String {
  let style = Style::new().fg(color);
  format!(
    "{}  {}: {value}",
    style.apply_to(icon),
    style.apply_to(label)
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn formats_binary_units_consistently() {
    assert_eq!(format_usage((1 << 30, 1 << 40)), "1.00 GiB / 1.00 TiB");
    assert_eq!(format_bytes(0), "0.00 B");
  }
}
