use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Status {
  Linked,
  Mismatch(PathBuf),
  Missing,
  Occupied,
  SourceMissing,
}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Linked => write!(f, "linked"),
      Self::Mismatch(destination) => {
        write!(f, "linked to `{}`", destination.display())
      }
      Self::Missing => write!(f, "missing"),
      Self::Occupied => write!(f, "occupied"),
      Self::SourceMissing => write!(f, "source missing"),
    }
  }
}

pub(crate) fn status(target: &Path, source: &Path) -> io::Result<Status> {
  if !source.try_exists()? {
    return Ok(Status::SourceMissing);
  }

  let metadata = match fs::symlink_metadata(target) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      return Ok(Status::Missing);
    }
    Err(error) => return Err(error),
  };

  if !metadata.is_symlink() {
    return Ok(Status::Occupied);
  }

  let destination = fs::read_link(target)?;

  if destination == source {
    Ok(Status::Linked)
  } else {
    Ok(Status::Mismatch(destination))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(status: Status, expected: &str) {
      assert_eq!(status.to_string(), expected);
    }

    case(Status::Linked, "linked");
    case(Status::Mismatch("foo".into()), "linked to `foo`");
    case(Status::Missing, "missing");
    case(Status::Occupied, "occupied");
    case(Status::SourceMissing, "source missing");
  }

  #[test]
  fn status() {
    let dir = TempDir::new().unwrap();

    let source = dir.path().join("foo");

    let target = dir.path().join("bar");

    let status = || super::status(&target, &source).unwrap();

    assert_eq!(status(), Status::SourceMissing);

    fs::write(&source, "").unwrap();

    assert_eq!(status(), Status::Missing);

    symlink(&source, &target).unwrap();

    assert_eq!(status(), Status::Linked);

    fs::remove_file(&target).unwrap();

    symlink(dir.path().join("baz"), &target).unwrap();

    assert_eq!(status(), Status::Mismatch(dir.path().join("baz")));

    fs::remove_file(&target).unwrap();

    fs::write(&target, "").unwrap();

    assert_eq!(status(), Status::Occupied);
  }
}
