use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[arg(long, default_value = "strew.toml")]
  config: PathBuf,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    let home = env::home_dir().context("failed to determine home directory")?;

    Config::load(&self.config, &home)?;

    Ok(())
  }
}
