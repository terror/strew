use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Link {
  pub(crate) source: PathBuf,
  pub(crate) target: PathBuf,
}
