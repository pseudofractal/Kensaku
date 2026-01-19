use std::process::Command;

/// Return used and total bytes on the filesystem containing `/`.
pub fn get_disk_usage() -> Option<(u64, u64)> {
  let output = Command::new("df")
    .args(["-Pk", "/"])
    .env("LC_ALL", "C")
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  parse_df(&String::from_utf8_lossy(&output.stdout))
}

fn parse_df(output: &str) -> Option<(u64, u64)> {
  let line = output.lines().nth(1)?;
  let fields: Vec<_> = line.split_whitespace().collect();
  // POSIX df -P reports 1024-byte blocks with -k.
  let total = fields.get(1)?.parse::<u64>().ok()?.checked_mul(1024)?;
  let used = fields.get(2)?.parse::<u64>().ok()?.checked_mul(1024)?;
  (total > 0 && used <= total).then_some((used, total))
}

#[cfg(test)]
mod tests {
  use super::parse_df;

  #[test]
  fn parses_root_filesystem_in_bytes() {
    let output = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/root 1048576 262144 786432 25% /\n";
    assert_eq!(parse_df(output), Some((268_435_456, 1_073_741_824)));
    assert_eq!(parse_df("invalid"), None);
    assert_eq!(parse_df("header\n/dev/root 10 11 0 100% /"), None);
  }
}
