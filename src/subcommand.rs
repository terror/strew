use super::*;

mod check;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Check,
}

impl Subcommand {
  pub(crate) fn run(self, config: &Config) -> Result {
    match self {
      Self::Check => check::run(config),
    }
  }
}
