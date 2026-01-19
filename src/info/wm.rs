use std::{env, process::Command};

pub fn get_window_manager() -> Option<String> {
  for variable in [
    "XDG_CURRENT_DESKTOP",
    "XDG_SESSION_DESKTOP",
    "DESKTOP_SESSION",
  ] {
    if let Ok(value) = env::var(variable) {
      if !value.trim().is_empty() {
        return Some(value);
      }
    }
  }

  let known_desktops = [
    "hyprland",
    "sway",
    "i3",
    "bspwm",
    "openbox",
    "fluxbox",
    "xmonad",
    "herbstluftwm",
    "awesome",
    "kwin_wayland",
    "kwin_x11",
    "mutter",
    "marco",
  ];
  let output = Command::new("ps")
    .args(["-u", &users::get_current_uid().to_string(), "-o", "comm="])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .find_map(|line| {
      let name = line.trim().to_lowercase();
      known_desktops.contains(&name.as_str()).then_some(name)
    })
}
