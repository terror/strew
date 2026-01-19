use super::*;

pub(crate) fn run(config: &Config) -> Result {
  if config.files.is_empty() {
    return Ok(());
  }

  let (linked, missing, conflicts) = config
    .files
    .iter()
    .map(|(name, entry)| {
      (
        name,
        config.resolve_path(&entry.source),
        config.resolve_path(&entry.target),
      )
    })
    .fold(
      (0, 0, 0),
      |(linked, missing, conflicts), (name, source, target)| match State::get(
        &source, &target,
      ) {
        State::Linked => {
          println!(
            "[linked] {name}: {} -> {}",
            target.display(),
            source.display()
          );

          (linked + 1, missing, conflicts)
        }
        State::Missing => {
          println!("[missing] {name}: {} (not created)", target.display());
          (linked, missing + 1, conflicts)
        }
        State::Conflict => {
          println!(
            "[conflict] {name}: {} exists but is not a symlink",
            target.display()
          );

          (linked, missing, conflicts + 1)
        }
        State::Misdirected { actual } => {
          println!(
            "[misdirected] {name}: {} -> {} (expected {})",
            target.display(),
            actual.display(),
            source.display()
          );

          (linked, missing, conflicts + 1)
        }
        State::SourceMissing => {
          println!(
            "[source missing] {name}: {} does not exist",
            source.display()
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
