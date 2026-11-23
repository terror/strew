use super::*;

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
  #[serde(skip)]
  pub(crate) base: PathBuf,
  pub(crate) clean: Vec<PathBuf>,
  pub(crate) create: Vec<PathBuf>,
  pub(crate) link: BTreeMap<PathBuf, PathBuf>,
  pub(crate) shell: Vec<Shell>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum Shell {
  Command(String),
  Described(String, String),
}

impl Config {
  pub(crate) fn load(path: &Path, home: &Path) -> Result<Self> {
    let text = fs::read_to_string(path)
      .with_context(|| format!("failed to read config `{}`", path.display()))?;

    let base = path
      .canonicalize()?
      .parent()
      .map(Path::to_owned)
      .context("config path has no parent")?;

    Self::parse(&text, base, home)
      .with_context(|| format!("failed to parse config `{}`", path.display()))
  }

  fn parse(text: &str, base: PathBuf, home: &Path) -> Result<Self> {
    let config = toml::from_str::<Self>(text)?;

    let resolve = |path| Self::resolve(&base, home, path);

    Ok(Self {
      clean: config.clean.into_iter().map(resolve).collect(),
      create: config.create.into_iter().map(resolve).collect(),
      link: config
        .link
        .into_iter()
        .map(|(target, source)| (resolve(target), resolve(source)))
        .collect(),
      shell: config.shell,
      base,
    })
  }

  fn resolve(base: &Path, home: &Path, path: PathBuf) -> PathBuf {
    match path.strip_prefix("~") {
      Ok(rest) if rest.as_os_str().is_empty() => home.to_owned(),
      Ok(rest) => home.join(rest),
      Err(_) => base.join(path),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, indoc::indoc, tempfile::TempDir};

  #[test]
  fn resolve() {
    #[track_caller]
    fn case(path: &str, expected: &str) {
      assert_eq!(
        Config::resolve(Path::new("/base"), Path::new("/home"), path.into()),
        Path::new(expected),
      );
    }

    case("~", "/home");
    case("~/foo", "/home/foo");
    case("~foo", "/base/~foo");
    case("foo", "/base/foo");
    case("/foo", "/foo");
  }

  #[test]
  fn parse() {
    assert_eq!(
      Config::parse(
        indoc! {r#"
          clean = ["~"]
          create = ["foo"]
          shell = ["bar", ["baz", "qux"]]

          [link]
          "~/foo" = "bar"
        "#},
        "/base".into(),
        Path::new("/home"),
      )
      .unwrap(),
      Config {
        base: "/base".into(),
        clean: vec!["/home".into()],
        create: vec!["/base/foo".into()],
        link: [("/home/foo".into(), "/base/bar".into())].into(),
        shell: vec![
          Shell::Command("bar".into()),
          Shell::Described("baz".into(), "qux".into()),
        ],
      },
    );
  }

  #[test]
  fn empty() {
    assert_eq!(
      Config::parse("", "/base".into(), Path::new("/home")).unwrap(),
      Config {
        base: "/base".into(),
        ..Config::default()
      },
    );
  }

  #[test]
  fn unknown_field() {
    assert_eq!(
      Config::parse("foo = []", "/base".into(), Path::new("/home"))
        .unwrap_err()
        .to_string(),
      indoc! {"
        TOML parse error at line 1, column 1
          |
        1 | foo = []
          | ^^^
        unknown field `foo`, expected one of `clean`, `create`, `link`, `shell`
      "},
    );
  }

  #[test]
  fn load() {
    let dir = TempDir::new().unwrap();

    let path = dir.path().join("foo.toml");

    fs::write(&path, "").unwrap();

    assert_eq!(
      Config::load(&path, Path::new("/home")).unwrap(),
      Config {
        base: dir.path().canonicalize().unwrap(),
        ..Config::default()
      },
    );
  }

  #[test]
  fn load_missing() {
    assert_eq!(
      format!(
        "{:#}",
        Config::load(Path::new("foo"), Path::new("/home")).unwrap_err()
      ),
      "failed to read config `foo`: No such file or directory (os error 2)",
    );
  }
}
