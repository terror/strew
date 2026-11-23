use super::*;

pub(crate) fn run(config: &Config, dry_run: bool) -> Result {
  let failures = config
    .clean
    .iter()
    .map(|directory| clean(directory, &config.base, dry_run))
    .chain(config.create.iter().map(|path| create(path, dry_run)))
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
      config.clean.len() + config.create.len() + config.link.len(),
    );
  }

  Ok(())
}

fn clean(directory: &Path, base: &Path, dry_run: bool) -> Result {
  let context = || format!("failed to clean `{}`", directory.display());

  let entries = match fs::read_dir(directory) {
    Ok(entries) => entries,
    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
    Err(error) => return Err(error).with_context(context),
  };

  let mut paths = entries
    .map(|entry| entry.map(|entry| entry.path()))
    .collect::<io::Result<Vec<PathBuf>>>()
    .with_context(context)?;

  paths.sort();

  for path in paths {
    if !path.is_symlink() || path.try_exists().with_context(context)? {
      continue;
    }

    let destination =
      directory.join(fs::read_link(&path).with_context(context)?);

    if !destination.starts_with(base) {
      continue;
    }

    if dry_run {
      println!("{}: would remove", path.display());
    } else {
      fs::remove_file(&path).with_context(context)?;
      println!("{}: removed", path.display());
    }
  }

  Ok(())
}

fn create(path: &Path, dry_run: bool) -> Result {
  let context = || format!("failed to create directory `{}`", path.display());

  match fs::symlink_metadata(path) {
    Ok(_) if path.is_dir() => {
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
