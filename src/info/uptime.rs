use std::fs;

pub fn get_uptime() -> Option<String> {
  let contents = fs::read_to_string("/proc/uptime").ok()?;
  let seconds: f64 = contents.split_whitespace().next()?.parse().ok()?;
  if !seconds.is_finite() || seconds < 0.0 {
    return None;
  }
  let minutes = seconds as u64 / 60;
  let days = minutes / (24 * 60);
  let hours = (minutes / 60) % 24;
  let minutes = minutes % 60;
  Some(if days > 0 {
    format!("{days}d {hours}h {minutes}m")
  } else {
    format!("{hours}h {minutes}m")
  })
}
