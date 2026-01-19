use std::process::Command;

pub fn get_package_count() -> Option<String> {
  let managers: &[(&str, &[&str])] = &[
    ("pacman", &["-Qq"]),
    ("dpkg-query", &["-W", "-f=${db:Status-Abbrev}\\n"]),
    ("rpm", &["-qa"]),
    ("flatpak", &["list", "--app", "--columns=application"]),
  ];
  let counts: Vec<_> = managers
    .iter()
    .filter_map(|(name, args)| {
      let output = Command::new(name)
        .args(*args)
        .env("LC_ALL", "C")
        .output()
        .ok()?;
      if !output.status.success() {
        return None;
      }
      let stdout = String::from_utf8_lossy(&output.stdout);
      let count = if *name == "dpkg-query" {
        stdout
          .lines()
          .filter(|line| line.starts_with("ii "))
          .count()
      } else {
        stdout
          .lines()
          .filter(|line| !line.trim().is_empty())
          .count()
      };
      Some(format!("{count} ({name})"))
    })
    .collect();
  (!counts.is_empty()).then(|| counts.join(", "))
}
