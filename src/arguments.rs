use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[arg(long, default_value = "strew.toml")]
  config: PathBuf,
  #[command(subcommand)]
  subcommand: Option<Subcommand>,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    let home = env::home_dir().context("failed to determine home directory")?;

    let config = Config::load(&self.config, &home)?;

    if let Some(subcommand) = self.subcommand {
      subcommand.run(&config)?;
    }

    Ok(())
  }
}
