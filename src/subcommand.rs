use super::*;

#[derive(Parser)]
pub(crate) enum Subcommand {
  /// Create symlinks for all configured entries
  Link,
  /// Show the status of all configured symlinks
  Status,
  /// Remove symlinks for all configured entries
  Unlink,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    todo!()
  }
}
