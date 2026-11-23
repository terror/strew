use super::*;

pub(crate) struct Plan<'a> {
  directories: BTreeSet<&'a Path>,
  pub(crate) items: Vec<Result<Vec<Action<'a>>>>,
  removed: BTreeSet<PathBuf>,
}

impl<'a> Plan<'a> {
  fn clean(
    &mut self,
    directory: &Path,
    base: &Path,
  ) -> Result<Vec<Action<'a>>> {
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

    let mut removals = Vec::new();

    for path in paths {
      if !path.is_symlink() || path.try_exists().with_context(context)? {
        continue;
      }

      let destination =
        directory.join(fs::read_link(&path).with_context(context)?);

      if destination.starts_with(base) {
        removals.push(path);
      }
    }

    self.removed.extend(removals.iter().cloned());

    Ok(removals.into_iter().map(Action::Remove).collect())
  }

  fn create(&mut self, path: &'a Path) -> Result<Action<'a>> {
    let context = || format!("failed to create directory `{}`", path.display());

    let status = if self.removed.contains(path) {
      directory::Status::Missing
    } else {
      directory::status(path).with_context(context)?
    };

    match status {
      directory::Status::Exists => Ok(Action::Exists(path)),
      directory::Status::Missing => {
        self.directories.insert(path);
        Ok(Action::CreateDirectory(path))
      }
      status => bail!("{}: {status}", context()),
    }
  }

  fn link(&self, target: &'a Path, source: &'a Path) -> Result<Action<'a>> {
    let context = || format!("failed to link `{}`", target.display());

    let status = match link::status(target, source).with_context(context)? {
      Status::Mismatch(_) if self.removed.contains(target) => Status::Missing,
      Status::Missing if self.directories.contains(target) => Status::Occupied,
      status => status,
    };

    match status {
      Status::Linked => Ok(Action::Linked(target)),
      Status::Missing => Ok(Action::CreateLink { source, target }),
      status => bail!("{}: {status}", context()),
    }
  }

  pub(crate) fn new(config: &'a Config, shell: bool) -> Self {
    let mut plan = Self {
      directories: BTreeSet::new(),
      items: Vec::new(),
      removed: BTreeSet::new(),
    };

    for directory in &config.clean {
      let item = plan.clean(directory, &config.base);
      plan.items.push(item);
    }

    for path in &config.create {
      let item = plan.create(path).map(|action| vec![action]);
      plan.items.push(item);
    }

    for (target, source) in &config.link {
      let item = plan.link(target, source).map(|action| vec![action]);
      plan.items.push(item);
    }

    if shell {
      plan.items.extend(
        config
          .shell
          .iter()
          .map(|shell| Ok(vec![Action::Run(shell)])),
      );
    }

    plan
  }
}
