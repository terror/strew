use super::*;

#[derive(Parser)]
pub(crate) struct Arguments {
  #[arg(long, default_value = "strew.toml")]
  config: PathBuf,
  #[arg(long)]
  dry_run: bool,
  #[arg(long, short)]
  verbose: bool,
  #[command(subcommand)]
  subcommand: Option<Subcommand>,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    let home = env::home_dir().context("failed to determine home directory")?;

    let config = Config::load(&self.config, &home)?;

    match self.subcommand {
      Some(subcommand) => subcommand.run(&config),
      None => apply::run(&config, self.dry_run, self.verbose),
    }
  }
}
