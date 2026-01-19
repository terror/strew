use super::*;

pub(crate) fn run(config: &Config) -> Result {
  let style = Style::stdout();

  println!("{}", style.apply(style::BOLD, "[link]"));

  for (name, link) in config.links() {
    match LinkState::from(link.clone()) {
      LinkState::Linked => {
        println!(
          "{} {} -> {}",
          style.apply(style::GREEN, name),
          style.apply(style::CYAN, link.target.display()),
          style.apply(style::CYAN, link.source.display())
        );
      }
      LinkState::Missing => {
        println!(
          "{} {} {}",
          style.apply(style::YELLOW, name),
          style.apply(style::CYAN, link.target.display()),
          style.apply(style::DIM, "(not created)")
        );
      }
      LinkState::Conflict => {
        println!(
          "{} {} {}",
          style.apply(style::RED, name),
          style.apply(style::CYAN, link.target.display()),
          style.apply(style::DIM, "exists but is not a symlink")
        );
      }
      LinkState::Misdirected { actual } => {
        println!(
          "{} {} -> {} {}",
          style.apply(style::RED, name),
          style.apply(style::CYAN, link.target.display()),
          style.apply(style::CYAN, actual.display()),
          style
            .apply(style::DIM, format!("(expected {})", link.source.display()))
        );
      }
      LinkState::SourceMissing => {
        println!(
          "{} {} {}",
          style.apply(style::RED, name),
          style.apply(style::CYAN, link.source.display()),
          style.apply(style::DIM, "does not exist")
        );
      }
    }
  }

  Ok(())
}
