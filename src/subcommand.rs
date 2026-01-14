use super::*;

mod status;

#[derive(Parser)]
pub(crate) enum Subcommand {
  /// Show the status of all configured symlinks
  Status,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    let config = Config::load()?;

    match self {
      Self::Status => status::run(&config),
    }
  }
}
