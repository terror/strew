use super::*;

pub(crate) fn run(config: &Config) -> Result {
  let mut failures = 0;

  for (target, source) in &config.link {
    let status = link::status(target, source).with_context(|| {
      format!("failed to check link `{}`", target.display())
    })?;

    if status != Status::Linked {
      failures += 1;
    }

    println!("{}: {status}", target.display());
  }

  if failures > 0 {
    bail!("{failures} of {} links not linked", config.link.len());
  }

  Ok(())
}
