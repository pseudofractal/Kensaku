use std::{
  fs,
  process::Command,
  time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn default_and_invalid_configuration() {
  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let directory =
    std::env::temp_dir().join(format!("kensaku-test-{}-{unique}", std::process::id()));
  fs::create_dir_all(directory.join("kensaku")).unwrap();
  let run = || {
    Command::new(env!("CARGO_BIN_EXE_kensaku"))
      .env("XDG_CONFIG_HOME", &directory)
      .output()
      .unwrap()
  };

  let default = run();
  assert!(default.status.success());
  assert!(String::from_utf8_lossy(&default.stdout).contains("Disk /"));

  fs::write(directory.join("kensaku/config.toml"), "disk = true\n").unwrap();
  let without_art = run();
  assert!(without_art.status.success());
  assert!(String::from_utf8_lossy(&without_art.stdout).starts_with('╭'));

  fs::write(directory.join("kensaku/config.toml"), "unknown = true\n").unwrap();
  let invalid = run();
  assert!(!invalid.status.success());
  assert!(String::from_utf8_lossy(&invalid.stderr).contains("unknown field"));
  let list = Command::new(env!("CARGO_BIN_EXE_kensaku"))
    .arg("--list-fractals")
    .env("XDG_CONFIG_HOME", &directory)
    .output()
    .unwrap();
  assert!(list.status.success());
  assert!(String::from_utf8_lossy(&list.stdout).contains("sierpinski_carpet"));

  fs::write(
    directory.join("kensaku/config.toml"),
    "[art]\nfractal = \"koch_curve\"\ndepth = 7\n",
  )
  .unwrap();
  let invalid_depth = run();
  assert!(!invalid_depth.status.success());
  assert!(String::from_utf8_lossy(&invalid_depth.stderr).contains("depth"));
  for (config, message) in [
    (
      "[art]\nfractal = \"julia_set\"\njulia_c = [nan, 0.0]\n",
      "julia_c",
    ),
    (
      "[art]\nfractal = \"multibrot\"\nmultibrot_power = 9\n",
      "multibrot_power",
    ),
    ("[art]\nmax_iterations = 0\n", "max_iterations"),
    ("[art]\njulia_c = [0.1]\n", "julia_c"),
  ] {
    fs::write(directory.join("kensaku/config.toml"), config).unwrap();
    let output = run();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(message));
  }
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn disabled_fields_do_not_run_collectors() {
  use std::os::unix::fs::PermissionsExt;

  let unique = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let directory = std::env::temp_dir().join(format!(
    "kensaku-collectors-{}-{unique}",
    std::process::id()
  ));
  let config_dir = directory.join("config/kensaku");
  let bin_dir = directory.join("bin");
  let marker = directory.join("invocations");
  fs::create_dir_all(&config_dir).unwrap();
  fs::create_dir_all(&bin_dir).unwrap();
  for command in ["df", "pacman", "dpkg-query", "rpm", "flatpak", "ps"] {
    let path = bin_dir.join(command);
    fs::write(
      &path,
      "#!/bin/sh\nprintf 'invoked\\n' >> \"$KENSAKU_TEST_MARKER\"\n",
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
  }
  let run = || {
    Command::new(env!("CARGO_BIN_EXE_kensaku"))
      .env("XDG_CONFIG_HOME", directory.join("config"))
      .env("PATH", &bin_dir)
      .env("KENSAKU_TEST_MARKER", &marker)
      .env_remove("XDG_CURRENT_DESKTOP")
      .env_remove("XDG_SESSION_DESKTOP")
      .env_remove("DESKTOP_SESSION")
      .output()
      .unwrap()
  };

  fs::write(
    config_dir.join("config.toml"),
    "disk = false\npackages = false\nwm = false\nip = false\n",
  )
  .unwrap();
  assert!(run().status.success());
  assert!(!marker.exists(), "disabled fields invoked a command");

  fs::write(
    config_dir.join("config.toml"),
    "disk = true\npackages = true\nwm = true\n",
  )
  .unwrap();
  assert!(run().status.success());
  assert_eq!(fs::read_to_string(&marker).unwrap().lines().count(), 6);
  fs::remove_dir_all(directory).unwrap();
}
