use super::*;

/// Represents the current state of a symlink entry
#[derive(Debug, PartialEq)]
pub(crate) enum EntryState {
  /// Target exists but is not a symlink (file or directory)
  Conflict,
  /// Symlink exists and points to the correct source
  Linked,
  /// Target is a symlink but points to wrong location
  Misdirected { actual: PathBuf },
  /// No symlink exists at target location
  Missing,
  /// Source path does not exist
  SourceMissing,
}

#[derive(Debug)]
pub(crate) struct Linker<'a> {
  config: &'a Config,
}

impl<'a> Linker<'a> {
  fn create_symlink(source: &Path, target: &Path) -> Result {
    if let Some(parent) = target.parent()
      && !parent.exists()
    {
      fs::create_dir_all(parent).with_context(|| {
        format!("failed to create parent directory: {}", parent.display())
      })?;
    }

    #[cfg(unix)]
    unix::fs::symlink(source, target).with_context(|| {
      format!(
        "failed to create symlink: {} -> {}",
        target.display(),
        source.display()
      )
    })?;

    #[cfg(windows)]
    {
      if source.is_dir() {
        windows::fs::symlink_dir(source, target).with_context(|| {
          format!(
            "failed to create symlink: {} -> {}",
            target.display(),
            source.display()
          )
        })?;
      } else {
        windows::fs::symlink_file(source, target).with_context(|| {
          format!(
            "failed to create symlink: {} -> {}",
            target.display(),
            source.display()
          )
        })?;
      }
    }

    Ok(())
  }

  fn expand_path(path: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(path).as_ref())
  }

  fn get_entry_state(source: &Path, target: &Path) -> EntryState {
    if !source.exists() {
      return EntryState::SourceMissing;
    }

    fs::symlink_metadata(target)
      .ok()
      .map_or(EntryState::Missing, |metadata| {
        if !metadata.file_type().is_symlink() {
          return EntryState::Conflict;
        }

        fs::read_link(target)
          .ok()
          .map_or(EntryState::Conflict, |link_target| {
            let canonical_source = source
              .canonicalize()
              .unwrap_or_else(|_| source.to_path_buf());

            let canonical_link = link_target
              .canonicalize()
              .unwrap_or_else(|_| link_target.clone());

            if canonical_source == canonical_link {
              EntryState::Linked
            } else {
              EntryState::Misdirected {
                actual: link_target,
              }
            }
          })
      })
  }

  pub(crate) fn link(&self) -> Result {
    if self.config.files.is_empty() {
      return Ok(());
    }

    let (created, skipped, errors) = self
      .config
      .files
      .iter()
      .map(|(name, entry)| {
        (
          name,
          Self::expand_path(&entry.source),
          Self::expand_path(&entry.target),
        )
      })
      .fold(
        (0, 0, 0),
        |(created, skipped, errors), (name, source, target)| {
          match Self::get_entry_state(&source, &target) {
            EntryState::Linked => {
              println!("  [skip] {name}: already linked");
              (created, skipped + 1, errors)
            }
            EntryState::Missing => Self::create_symlink(&source, &target)
              .map_or_else(
                |error| {
                  eprintln!(
                    "  [error] {name}: failed to create symlink: {error}"
                  );
                  (created, skipped, errors + 1)
                },
                |()| {
                  println!(
                    "  [link] {name}: {} -> {}",
                    target.display(),
                    source.display()
                  );

                  (created + 1, skipped, errors)
                },
              ),
            EntryState::Conflict => {
              eprintln!(
                "  [conflict] {name}: {} exists and is not a symlink",
                target.display()
              );

              (created, skipped, errors + 1)
            }
            EntryState::Misdirected { actual } => {
              eprintln!(
                "  [conflict] {name}: {} points to {} instead of {}",
                target.display(),
                actual.display(),
                source.display()
              );

              (created, skipped, errors + 1)
            }
            EntryState::SourceMissing => {
              eprintln!(
                "  [error] {name}: source {} does not exist",
                source.display()
              );

              (created, skipped, errors + 1)
            }
          }
        },
      );

    println!("Summary: {created} created, {skipped} skipped, {errors} errors");

    ensure!(errors == 0, "{errors} error(s) occurred during linking");

    Ok(())
  }

  pub(crate) fn new(config: &'a Config) -> Self {
    Self { config }
  }

  pub(crate) fn status(&self) -> Result {
    if self.config.files.is_empty() {
      return Ok(());
    }

    let (linked, missing, conflicts) = self
      .config
      .files
      .iter()
      .map(|(name, entry)| {
        (
          name,
          Self::expand_path(&entry.source),
          Self::expand_path(&entry.target),
        )
      })
      .fold(
        (0, 0, 0),
        |(linked, missing, conflicts), (name, source, target)| {
          match Self::get_entry_state(&source, &target) {
            EntryState::Linked => {
              println!(
                "  [linked] {name}: {} -> {}",
                target.display(),
                source.display()
              );

              (linked + 1, missing, conflicts)
            }
            EntryState::Missing => {
              println!(
                "  [missing] {name}: {} (not created)",
                target.display()
              );
              (linked, missing + 1, conflicts)
            }
            EntryState::Conflict => {
              println!(
                "  [conflict] {name}: {} exists but is not a symlink",
                target.display()
              );

              (linked, missing, conflicts + 1)
            }
            EntryState::Misdirected { actual } => {
              println!(
                "  [misdirected] {name}: {} -> {} (expected {})",
                target.display(),
                actual.display(),
                source.display()
              );

              (linked, missing, conflicts + 1)
            }
            EntryState::SourceMissing => {
              println!(
                "  [source missing] {name}: {} does not exist",
                source.display()
              );

              (linked, missing, conflicts + 1)
            }
          }
        },
      );

    println!(
      "Summary: {linked} linked, {missing} missing, {conflicts} conflicts"
    );

    Ok(())
  }

  pub(crate) fn unlink(&self) -> Result {
    if self.config.files.is_empty() {
      return Ok(());
    }

    let (removed, skipped, errors) = self
      .config
      .files
      .iter()
      .map(|(name, entry)| {
        (
          name,
          Self::expand_path(&entry.source),
          Self::expand_path(&entry.target),
        )
      })
      .fold(
        (0, 0, 0),
        |(removed, skipped, errors), (name, source, target)| {
          match Self::get_entry_state(&source, &target) {
            EntryState::Linked => fs::remove_file(&target).map_or_else(
              |error| {
                eprintln!(
                  "  [error] {name}: failed to remove symlink: {error}"
                );
                (removed, skipped, errors + 1)
              },
              |()| {
                println!("  [unlink] {name}: removed {}", target.display());
                (removed + 1, skipped, errors)
              },
            ),
            EntryState::Missing | EntryState::SourceMissing => {
              println!("  [skip] {name}: not linked");
              (removed, skipped + 1, errors)
            }
            EntryState::Conflict => {
              eprintln!(
                "  [skip] {name}: {} is not a symlink, refusing to remove",
                target.display()
              );

              (removed, skipped + 1, errors)
            }
            EntryState::Misdirected { actual } => {
              eprintln!(
                "  [skip] {name}: {} points to {}, not managed by strew",
                target.display(),
                actual.display()
              );

              (removed, skipped + 1, errors)
            }
          }
        },
      );

    println!("Summary: {removed} removed, {skipped} skipped, {errors} errors");

    ensure!(errors == 0, "{errors} error(s) occurred during unlinking");

    Ok(())
  }
}
