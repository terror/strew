use {
  anyhow::Error,
  indoc::indoc,
  pretty_assertions::assert_eq,
  std::{fs, os::unix::fs::symlink, process::Command, str},
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

struct Test<'a> {
  arguments: Vec<String>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  files: Vec<(&'a str, &'a str)>,
  symlinks: Vec<(&'a str, &'a str)>,
  tempdir: TempDir,
}

impl<'a> Test<'a> {
  fn argument(mut self, argument: &str) -> Self {
    self.arguments.push(argument.to_owned());

    self
  }

  fn config(self, config: &'a str) -> Self {
    self.file("strew.toml", config)
  }

  fn expected_status(mut self, expected_status: i32) -> Self {
    self.expected_status = expected_status;

    self
  }

  fn expected_stderr(mut self, expected_stderr: &str) -> Self {
    expected_stderr.clone_into(&mut self.expected_stderr);

    self
  }

  fn expected_stdout(mut self, expected_stdout: &str) -> Self {
    expected_stdout.clone_into(&mut self.expected_stdout);

    self
  }

  fn file(mut self, path: &'a str, content: &'a str) -> Self {
    self.files.push((path, content));

    self
  }

  fn new() -> Result<Self> {
    Ok(Self {
      arguments: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      files: Vec::new(),
      symlinks: Vec::new(),
      tempdir: TempDir::new()?,
    })
  }

  fn run(self) -> Result {
    let root = self.tempdir.path().canonicalize()?;

    for (path, content) in &self.files {
      fs::write(root.join(path), content)?;
    }

    for (target, source) in &self.symlinks {
      symlink(root.join(source), root.join(target))?;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_strew"))
      .args(&self.arguments)
      .current_dir(&root)
      .env("HOME", &root)
      .output()?;

    let root = root.display().to_string();

    let stderr = str::from_utf8(&output.stderr)?.replace(&root, "[ROOT]");

    assert_eq!(
      output.status.code(),
      Some(self.expected_status),
      "unexpected exit status\nstderr: {stderr}"
    );

    assert_eq!(stderr, self.expected_stderr);

    assert_eq!(
      str::from_utf8(&output.stdout)?.replace(&root, "[ROOT]"),
      self.expected_stdout,
    );

    Ok(())
  }

  fn symlink(mut self, target: &'a str, source: &'a str) -> Self {
    self.symlinks.push((target, source));

    self
  }
}

#[test]
fn check() -> Result {
  Test::new()?
    .config(indoc! {r#"
      [link]
      "~/bar" = "foo"
      "~/baz" = "foo"
    "#})
    .file("foo", "")
    .symlink("bar", "foo")
    .argument("check")
    .expected_status(1)
    .expected_stdout(indoc! {"
      [ROOT]/bar: linked
      [ROOT]/baz: missing
    "})
    .expected_stderr("error: 1 of 2 links not linked\n")
    .run()
}
