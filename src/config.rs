use super::*;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Config {
  #[serde(skip)]
  pub(crate) base_dir: Option<PathBuf>,
  #[serde(default)]
  pub(crate) files: HashMap<String, Entry>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Entry {
  pub(crate) source: String,
  pub(crate) target: String,
}

impl Config {
  const APP_NAME: &'static str = "strew";
  const CONFIG_NAME: &'static str = "config";

  pub(crate) fn load() -> Result<Self> {
    let config_path =
      confy::get_configuration_file_path(Self::APP_NAME, Self::CONFIG_NAME)?;

    let base_dir = fs::canonicalize(&config_path)
      .unwrap_or(config_path)
      .parent()
      .map(Path::to_path_buf);

    let config = confy::load::<Self>(Self::APP_NAME, Self::CONFIG_NAME)?;

    Ok(Self { base_dir, ..config })
  }

  pub(crate) fn resolve_path(&self, path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path);

    let path = PathBuf::from(expanded.as_ref());

    if path.is_absolute() {
      path
    } else {
      self
        .base_dir
        .as_ref()
        .map(|base| base.join(&path))
        .unwrap_or(path)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolve_path_absolute() {
    let config = Config {
      base_dir: Some(PathBuf::from("/config/dir")),
      ..Default::default()
    };

    assert_eq!(config.resolve_path("/absolute/path"), PathBuf::from("/absolute/path"));
  }

  #[test]
  fn resolve_path_relative_with_base_dir() {
    let config = Config {
      base_dir: Some(PathBuf::from("/config/dir")),
      ..Default::default()
    };

    assert_eq!(
      config.resolve_path("relative/path"),
      PathBuf::from("/config/dir/relative/path")
    );
  }

  #[test]
  fn resolve_path_relative_without_base_dir() {
    let config = Config {
      base_dir: None,
      ..Default::default()
    };

    assert_eq!(config.resolve_path("relative/path"), PathBuf::from("relative/path"));
  }

  #[test]
  fn resolve_path_tilde_expansion() {
    let config = Config {
      base_dir: Some(PathBuf::from("/config/dir")),
      ..Default::default()
    };

    let resolved = config.resolve_path("~/some/path");

    assert!(resolved.is_absolute());
    assert!(resolved.ends_with("some/path"));
  }
}
