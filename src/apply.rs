use super::*;

pub(crate) fn run(config: &Config, dry_run: bool, verbose: bool) -> Result {
  let mut removed = BTreeSet::new();

  let mut failures = config
    .clean
    .iter()
    .map(|directory| {
      clean(directory, &config.base, dry_run).map(|paths| removed.extend(paths))
    })
    .filter_map(Result::err)
    .inspect(report)
    .count();

  failures +=
    config
      .create
      .iter()
      .map(|path| create(path, dry_run, verbose))
      .chain(config.link.iter().map(|(target, source)| {
        link(target, source, &removed, dry_run, verbose)
      }))
      .chain(
        config
          .shell
          .iter()
          .map(|shell| run_shell(shell, &config.base, dry_run)),
      )
      .filter_map(Result::err)
      .inspect(report)
      .count();

  if failures > 0 {
    bail!(
      "{failures} of {} items failed",
      config.clean.len()
        + config.create.len()
        + config.link.len()
        + config.shell.len(),
    );
  }

  Ok(())
}

fn clean(directory: &Path, base: &Path, dry_run: bool) -> Result<Vec<PathBuf>> {
  let context = || format!("failed to clean `{}`", directory.display());

  let entries = match fs::read_dir(directory) {
    Ok(entries) => entries,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      return Ok(Vec::new());
    }
    Err(error) => return Err(error).with_context(context),
  };

  let mut paths = entries
    .map(|entry| entry.map(|entry| entry.path()))
    .collect::<io::Result<Vec<PathBuf>>>()
    .with_context(context)?;

  paths.sort();

  let mut removed = Vec::new();

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

    removed.push(path);
  }

  Ok(removed)
}

fn create(path: &Path, dry_run: bool, verbose: bool) -> Result {
  let context = || format!("failed to create directory `{}`", path.display());

  match directory::status(path).with_context(context)? {
    directory::Status::Exists if verbose => {
      println!("{}: exists", path.display());
    }
    directory::Status::Exists => {}
    directory::Status::Missing if dry_run => {
      println!("{}: would create", path.display());
    }
    directory::Status::Missing => {
      fs::create_dir_all(path).with_context(context)?;
      println!("{}: created", path.display());
    }
    status => bail!("{}: {status}", context()),
  }

  Ok(())
}

fn link(
  target: &Path,
  source: &Path,
  removed: &BTreeSet<PathBuf>,
  dry_run: bool,
  verbose: bool,
) -> Result {
  let context = || format!("failed to link `{}`", target.display());

  let status = match link::status(target, source).with_context(context)? {
    Status::Mismatch(_) if removed.contains(target) => Status::Missing,
    status => status,
  };

  match status {
    Status::Linked if verbose => println!("{}: linked", target.display()),
    Status::Linked => {}
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

fn run_shell(shell: &Shell, base: &Path, dry_run: bool) -> Result {
  if dry_run {
    println!("{}: would run", shell.label());
    return Ok(());
  }

  println!("{}: running", shell.label());

  let context = || format!("failed to run `{}`", shell.command());

  let status = Command::new("sh")
    .arg("-c")
    .arg(shell.command())
    .current_dir(base)
    .status()
    .with_context(context)?;

  if !status.success() {
    bail!("{}: {status}", context());
  }

  Ok(())
}

fn report(error: &anyhow::Error) {
  eprintln!("error: {error:#}");
}
