use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Action<'a> {
  CreateDirectory(&'a Path),
  CreateLink { source: &'a Path, target: &'a Path },
  Exists(&'a Path),
  Linked(&'a Path),
  Remove(PathBuf),
  Run(&'a Shell),
}

impl Action<'_> {
  pub(crate) fn changes(&self) -> bool {
    !matches!(self, Self::Exists(_) | Self::Linked(_))
  }

  pub(crate) fn execute(&self, base: &Path) -> Result {
    match self {
      Self::CreateDirectory(path) => {
        fs::create_dir_all(path).with_context(|| {
          format!("failed to create directory `{}`", path.display())
        })
      }
      Self::CreateLink { source, target } => link::create(target, source)
        .with_context(|| format!("failed to link `{}`", target.display())),
      Self::Exists(_) | Self::Linked(_) => Ok(()),
      Self::Remove(path) => fs::remove_file(path)
        .with_context(|| format!("failed to remove `{}`", path.display())),
      Self::Run(shell) => {
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
    }
  }

  pub(crate) fn print(&self, dry_run: bool) {
    let subject = match self {
      Self::CreateDirectory(path) | Self::Exists(path) | Self::Linked(path) => {
        path.display().to_string()
      }
      Self::CreateLink { target, .. } => target.display().to_string(),
      Self::Remove(path) => path.display().to_string(),
      Self::Run(shell) => shell.label().to_owned(),
    };

    let status = match (self, dry_run) {
      (Self::CreateDirectory(_) | Self::CreateLink { .. }, true) => {
        "would create"
      }
      (Self::CreateDirectory(_) | Self::CreateLink { .. }, false) => "created",
      (Self::Exists(_), _) => "exists",
      (Self::Linked(_), _) => "linked",
      (Self::Remove(_), true) => "would remove",
      (Self::Remove(_), false) => "removed",
      (Self::Run(_), true) => "would run",
      (Self::Run(_), false) => "running",
    };

    println!("{subject}: {status}");
  }
}
