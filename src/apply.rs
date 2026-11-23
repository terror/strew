use super::*;

pub(crate) fn run(config: &Config, dry_run: bool) -> Result {
  let failures = config
    .create
    .iter()
    .map(|path| create(path, dry_run))
    .chain(
      config
        .link
        .iter()
        .map(|(target, source)| link(target, source, dry_run)),
    )
    .filter_map(Result::err)
    .inspect(|error| eprintln!("error: {error:#}"))
    .count();

  if failures > 0 {
    bail!(
      "{failures} of {} items failed",
      config.create.len() + config.link.len(),
    );
  }

  Ok(())
}

fn create(path: &Path, dry_run: bool) -> Result {
  let context = || format!("failed to create directory `{}`", path.display());

  match fs::metadata(path) {
    Ok(metadata) if metadata.is_dir() => {
      println!("{}: exists", path.display());
    }
    Ok(_) => bail!("{}: occupied", context()),
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      if dry_run {
        println!("{}: would create", path.display());
      } else {
        fs::create_dir_all(path).with_context(context)?;
        println!("{}: created", path.display());
      }
    }
    Err(error) => return Err(error).with_context(context),
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
