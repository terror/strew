use super::*;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Config {
  #[serde(skip)]
  pub(crate) base_dir: Option<PathBuf>,
  #[serde(default)]
  pub(crate) link: BTreeMap<String, Link>,
}

impl Config {
  const APP_NAME: &'static str = "strew";
  const CONFIG_NAME: &'static str = "config";

  pub(crate) fn links(&self) -> Vec<(&str, Link)> {
    self
      .link
      .iter()
      .map(|(name, link)| {
        (
          name.as_str(),
          Link {
            source: self.resolve_path(&link.source),
            target: self.resolve_path(&link.target),
          },
        )
      })
      .collect()
  }

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

  pub(crate) fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
    let path = PathBuf::from(
      shellexpand::tilde(&path.as_ref().display().to_string()).as_ref(),
    );

    if path.is_absolute() {
      return path;
    }

    self
      .base_dir
      .as_ref()
      .map(|base| base.join(&path))
      .unwrap_or(path)
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

    assert_eq!(
      config.resolve_path("/absolute/path"),
      PathBuf::from("/absolute/path")
    );
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

    assert_eq!(
      config.resolve_path("relative/path"),
      PathBuf::from("relative/path")
    );
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
