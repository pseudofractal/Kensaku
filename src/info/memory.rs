use std::fs;

pub fn get_memory_usage() -> Option<(u64, u64)> {
  memory_usage(
    &fs::read_to_string("/proc/meminfo").ok()?,
    "MemTotal:",
    "MemAvailable:",
  )
}

pub fn get_swap_usage() -> Option<(u64, u64)> {
  memory_usage(
    &fs::read_to_string("/proc/meminfo").ok()?,
    "SwapTotal:",
    "SwapFree:",
  )
}

fn memory_usage(contents: &str, total_key: &str, available_key: &str) -> Option<(u64, u64)> {
  let value = |key| {
    contents.lines().find_map(|line| {
      let (name, rest) = line.split_once(':')?;
      if format!("{name}:") != key {
        return None;
      }
      rest.split_whitespace().next()?.parse::<u64>().ok()
    })
  };
  let total = value(total_key)?;
  let available = value(available_key)?;
  Some((
    total.checked_sub(available)?.checked_mul(1024)?,
    total.checked_mul(1024)?,
  ))
}

#[cfg(test)]
mod tests {
  use super::memory_usage;
  #[test]
  fn parses_memory_and_rejects_missing_values() {
    let input = "MemTotal: 2048 kB\nMemAvailable: 512 kB\n";
    assert_eq!(
      memory_usage(input, "MemTotal:", "MemAvailable:"),
      Some((1_572_864, 2_097_152))
    );
    assert_eq!(memory_usage(input, "SwapTotal:", "SwapFree:"), None);
  }
}
