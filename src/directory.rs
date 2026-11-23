use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Status {
  Exists,
  Missing,
  Occupied,
}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Exists => write!(f, "exists"),
      Self::Missing => write!(f, "missing"),
      Self::Occupied => write!(f, "occupied"),
    }
  }
}

pub(crate) fn status(path: &Path) -> io::Result<Status> {
  match fs::symlink_metadata(path) {
    Ok(_) if path.is_dir() => Ok(Status::Exists),
    Ok(_) => Ok(Status::Occupied),
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      Ok(Status::Missing)
    }
    Err(error) => Err(error),
  }
}
