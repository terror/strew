use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum LinkState {
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

impl From<Link> for LinkState {
  fn from(link: Link) -> Self {
    if !link.source.exists() {
      return Self::SourceMissing;
    }

    fs::symlink_metadata(&link.target)
      .ok()
      .map_or(Self::Missing, |metadata| {
        if !metadata.file_type().is_symlink() {
          return Self::Conflict;
        }

        fs::read_link(link.target)
          .ok()
          .map_or(Self::Conflict, |link_target| {
            let canonical_source = link
              .source
              .canonicalize()
              .unwrap_or_else(|_| link.source.clone());

            let canonical_link = link_target
              .canonicalize()
              .unwrap_or_else(|_| link_target.clone());

            if canonical_source == canonical_link {
              Self::Linked
            } else {
              Self::Misdirected {
                actual: link_target,
              }
            }
          })
      })
  }
}

#[cfg(all(test, unix))]
mod tests {
  use {super::*, std::os::unix::fs::symlink, tempfile::TempDir};

  #[test]
  fn source_missing() {
    let directory = TempDir::new().unwrap();

    assert_eq!(
      LinkState::from(Link {
        source: directory.path().join("nonexistent"),
        target: directory.path().join("link")
      }),
      LinkState::SourceMissing
    );
  }

  #[test]
  fn target_missing() {
    let directory = TempDir::new().unwrap();

    let source = directory.path().join("source");

    fs::write(&source, "content").unwrap();

    assert_eq!(
      LinkState::from(Link {
        source,
        target: directory.path().join("link")
      }),
      LinkState::Missing
    );
  }

  #[test]
  fn conflict_with_file() {
    let directory = TempDir::new().unwrap();

    let source = directory.path().join("source");
    let target = directory.path().join("target");

    fs::write(&source, "source content").unwrap();
    fs::write(&target, "target content").unwrap();

    assert_eq!(
      LinkState::from(Link { source, target }),
      LinkState::Conflict
    );
  }

  #[test]
  fn conflict_with_directory() {
    let directory = TempDir::new().unwrap();

    let source = directory.path().join("source");
    let target = directory.path().join("target");

    fs::write(&source, "content").unwrap();
    fs::create_dir(&target).unwrap();

    assert_eq!(
      LinkState::from(Link { source, target }),
      LinkState::Conflict
    );
  }

  #[test]
  fn linked() {
    let directory = TempDir::new().unwrap();

    let source = directory.path().join("source");
    let target = directory.path().join("link");

    fs::write(&source, "content").unwrap();

    symlink(&source, &target).unwrap();

    assert_eq!(LinkState::from(Link { source, target }), LinkState::Linked);
  }

  #[test]
  fn misdirected() {
    let directory = TempDir::new().unwrap();

    let source = directory.path().join("source");
    let other = directory.path().join("other");
    let target = directory.path().join("link");

    fs::write(&source, "content").unwrap();
    fs::write(&other, "other content").unwrap();

    symlink(&other, &target).unwrap();

    assert_eq!(
      LinkState::from(Link { source, target }),
      LinkState::Misdirected { actual: other }
    );
  }
}
