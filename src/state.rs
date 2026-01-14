use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum State {
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

impl State {
  pub(crate) fn get(source: &Path, target: &Path) -> Self {
    if !source.exists() {
      return Self::SourceMissing;
    }

    fs::symlink_metadata(target)
      .ok()
      .map_or(Self::Missing, |metadata| {
        if !metadata.file_type().is_symlink() {
          return Self::Conflict;
        }

        fs::read_link(target)
          .ok()
          .map_or(Self::Conflict, |link_target| {
            let canonical_source = source
              .canonicalize()
              .unwrap_or_else(|_| source.to_path_buf());

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
