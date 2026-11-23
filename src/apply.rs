use super::*;

pub(crate) fn run(config: &Config, dry_run: bool) -> Result {
  let mut failures = 0;

  for (target, source) in &config.link {
    if let Err(error) = link(target, source, dry_run) {
      eprintln!("error: {error:#}");
      failures += 1;
    }
  }

  if failures > 0 {
    bail!("{failures} of {} links failed", config.link.len());
  }

  Ok(())
}

fn link(target: &Path, source: &Path, dry_run: bool) -> Result {
  let context = || format!("failed to link `{}`", target.display());

  match link::status(target, source).with_context(context)? {
    Status::Linked => println!("{}: linked", target.display()),
    Status::Missing if dry_run => {
      println!("{}: would create", target.display());
    }
    Status::Missing => {
      link::create(target, source).with_context(context)?;
      println!("{}: created", target.display());
    }
    status => bail!("{}: {status}", context()),
  }

  Ok(())
}
