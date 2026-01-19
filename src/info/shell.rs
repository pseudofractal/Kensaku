use std::{env, path::Path};

pub fn get_shell() -> Option<String> {
  let shell = env::var_os("SHELL")?;
  let name = Path::new(&shell).file_name()?.to_string_lossy();
  (!name.is_empty()).then(|| name.into_owned())
}
