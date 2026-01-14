use super::*;

#[derive(Parser)]
#[command(name = "strew", about = "A tool to manage your dotfiles")]
pub(crate) struct Arguments {
  #[command(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    self.subcommand.run()
  }
}
