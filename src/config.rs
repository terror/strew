use super::*;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Config {
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

  pub(crate) fn load() -> Result<Self> {
    confy::load(Self::APP_NAME, None).context("failed to load config")
  }
}
