use super::*;

pub(crate) fn run(config: &Config) -> Result {
  let files = config.files();

  if files.is_empty() {
    return Ok(());
  }

  let style = Style::stdout();

  let (linked, missing, conflicts) = files.iter().fold(
    (0, 0, 0),
    |(linked, missing, conflicts), (name, source, target)| match State::get(
      source, target,
    ) {
      State::Linked => {
        println!(
          "{} {}: {} -> {}",
          style.apply(style::GREEN, "[linked]"),
          style.apply(style::BOLD, name),
          style.apply(style::CYAN, target.display()),
          style.apply(style::CYAN, source.display())
        );

        (linked + 1, missing, conflicts)
      }
      State::Missing => {
        println!(
          "{} {}: {} {}",
          style.apply(style::YELLOW, "[missing]"),
          style.apply(style::BOLD, name),
          style.apply(style::CYAN, target.display()),
          style.apply(style::DIM, "(not created)")
        );
        (linked, missing + 1, conflicts)
      }
      State::Conflict => {
        println!(
          "{} {}: {} {}",
          style.apply(style::RED, "[conflict]"),
          style.apply(style::BOLD, name),
          style.apply(style::CYAN, target.display()),
          style.apply(style::DIM, "exists but is not a symlink")
        );

        (linked, missing, conflicts + 1)
      }
      State::Misdirected { actual } => {
        println!(
          "{} {}: {} -> {} {}",
          style.apply(style::RED, "[misdirected]"),
          style.apply(style::BOLD, name),
          style.apply(style::CYAN, target.display()),
          style.apply(style::CYAN, actual.display()),
          style.apply(style::DIM, format!("(expected {})", source.display()))
        );

        (linked, missing, conflicts + 1)
      }
      State::SourceMissing => {
        println!(
          "{} {}: {} {}",
          style.apply(style::RED, "[source missing]"),
          style.apply(style::BOLD, name),
          style.apply(style::CYAN, source.display()),
          style.apply(style::DIM, "does not exist")
        );

        (linked, missing, conflicts + 1)
      }
    },
  );

  println!(
    "Summary: {linked} linked, {missing} missing, {conflicts} conflicts"
  );

  Ok(())
}
