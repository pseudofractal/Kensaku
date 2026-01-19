use std::{env, fs};

pub fn load_average() -> Option<String> {
  let contents = fs::read_to_string("/proc/loadavg").ok()?;
  let values: Vec<_> = contents.split_whitespace().take(3).collect();
  (values.len() == 3).then(|| values.join(" "))
}

pub fn cpu_threads() -> Option<String> {
  std::thread::available_parallelism()
    .ok()
    .map(|n| n.to_string())
}

pub fn terminal() -> Option<String> {
  env::var("TERM_PROGRAM")
    .or_else(|_| env::var("TERM"))
    .ok()
    .filter(|value| !value.is_empty())
}

pub fn host_model() -> Option<String> {
  let vendor = fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor").ok()?;
  let product = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").ok()?;
  let model = format!("{} {}", vendor.trim(), product.trim());
  (!vendor.trim().is_empty() && !product.trim().is_empty()).then_some(model)
}
