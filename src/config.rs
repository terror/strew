use super::*;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
  #[serde(default)]
  pub(crate) files: HashMap<String, Entry>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Entry {
  pub(crate) source: String,
  pub(crate) target: String,
}

impl Config {
  const APP_DIR: &'static str = "strew";
  const FILENAME: &'static str = "strew.toml";

  pub(crate) fn load() -> Result<Self> {
    Self::load_from(&Self::path()?)
  }

  fn load_from(path: &PathBuf) -> Result<Self> {
    let contents = fs::read_to_string(path).with_context(|| {
      format!("failed to read config file: {}", path.display())
    })?;

    toml::from_str(&contents).with_context(|| {
      format!("failed to parse config file: {}", path.display())
    })
  }

  fn path() -> Result<PathBuf> {
    Ok(
      dirs::config_dir()
        .context("could not determine config directory")?
        .join(Self::APP_DIR)
        .join(Self::FILENAME),
    )
  }
}
