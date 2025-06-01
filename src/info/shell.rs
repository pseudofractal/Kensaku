use std::env;
use std::process::Command;

pub fn get_shell() -> String {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let output = Command::new(&shell)
        .arg("--version")
        .output()
        .unwrap_or_else(|_| panic!("failed to get shell version"));

    let version = String::from_utf8_lossy(&output.stdout);
    let line = version.lines().next().unwrap_or("");

    let mut chars = line.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

