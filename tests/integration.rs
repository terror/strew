use {
  anyhow::Error,
  indoc::indoc,
  pretty_assertions::assert_eq,
  std::{fs, os::unix::fs::symlink, process::Command, str},
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

struct Test<'a> {
  absent: Vec<&'a str>,
  arguments: Vec<String>,
  directories: Vec<&'a str>,
  expected_directories: Vec<&'a str>,
  expected_links: Vec<(&'a str, &'a str)>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  files: Vec<(&'a str, &'a str)>,
  symlinks: Vec<(&'a str, &'a str)>,
  tempdir: TempDir,
}

impl<'a> Test<'a> {
  fn absent(mut self, path: &'a str) -> Self {
    self.absent.push(path);

    self
  }

  fn argument(mut self, argument: &str) -> Self {
    self.arguments.push(argument.to_owned());

    self
  }

  fn config(self, config: &'a str) -> Self {
    self.file("strew.toml", config)
  }

  fn directory(mut self, path: &'a str) -> Self {
    self.directories.push(path);

    self
  }

  fn expected_directory(mut self, path: &'a str) -> Self {
    self.expected_directories.push(path);

    self
  }

  fn expected_link(mut self, target: &'a str, source: &'a str) -> Self {
    self.expected_links.push((target, source));

    self
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
      absent: Vec::new(),
      arguments: Vec::new(),
      directories: Vec::new(),
      expected_directories: Vec::new(),
      expected_links: Vec::new(),
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

    for path in &self.directories {
      fs::create_dir(root.join(path))?;
    }

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

    for path in &self.absent {
      assert!(
        fs::symlink_metadata(root.join(path)).is_err(),
        "expected `{path}` to be absent"
      );
    }

    for path in &self.expected_directories {
      assert!(
        root.join(path).is_dir(),
        "expected `{path}` to be a directory"
      );
    }

    for (target, source) in &self.expected_links {
      assert_eq!(fs::read_link(root.join(target))?, root.join(source));
    }

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
fn apply() -> Result {
  Test::new()?
    .config(indoc! {r#"
      [link]
      "~/bar" = "foo"
      "~/baz/qux" = "foo"
    "#})
    .file("foo", "")
    .symlink("bar", "foo")
    .expected_link("baz/qux", "foo")
    .expected_stdout("[ROOT]/baz/qux: created\n")
    .run()
}

#[test]
fn apply_failure() -> Result {
  Test::new()?
    .config(indoc! {r#"
      [link]
      "~/bar" = "foo"
      "~/baz" = "foo"
    "#})
    .file("foo", "")
    .file("bar", "")
    .expected_link("baz", "foo")
    .expected_status(1)
    .expected_stdout("[ROOT]/baz: created\n")
    .expected_stderr(indoc! {"
      error: failed to link `[ROOT]/bar`: occupied
      error: 1 of 2 items failed
    "})
    .run()
}

#[test]
fn check() -> Result {
  Test::new()?
    .config(indoc! {r#"
      clean = ["~"]
      create = ["~/qux", "~/quux"]

      [link]
      "~/bar" = "foo"
      "~/baz" = "foo"
    "#})
    .directory("qux")
    .file("foo", "")
    .symlink("bar", "foo")
    .symlink("corge", "grault")
    .argument("check")
    .expected_status(1)
    .expected_link("corge", "grault")
    .expected_stdout(indoc! {"
      [ROOT]/corge: would remove
      [ROOT]/qux: exists
      [ROOT]/quux: would create
      [ROOT]/bar: linked
      [ROOT]/baz: would create
    "})
    .expected_stderr("error: 3 of 5 items need attention\n")
    .run()
}

#[test]
fn clean() -> Result {
  Test::new()?
    .directory("sub")
    .file("sub/strew.toml", r#"clean = ["~", "~/missing"]"#)
    .symlink("bar", "sub/baz")
    .symlink("qux", "quux")
    .argument("--config")
    .argument("sub/strew.toml")
    .absent("bar")
    .expected_link("qux", "quux")
    .expected_stdout("[ROOT]/bar: removed\n")
    .run()
}

#[test]
fn create() -> Result {
  Test::new()?
    .config(r#"create = ["~/foo", "~/bar/baz"]"#)
    .directory("foo")
    .expected_directory("bar/baz")
    .expected_stdout("[ROOT]/bar/baz: created\n")
    .run()
}

#[test]
fn create_failure() -> Result {
  Test::new()?
    .config(r#"create = ["~/foo", "~/bar"]"#)
    .symlink("foo", "baz")
    .expected_directory("bar")
    .expected_status(1)
    .expected_stdout("[ROOT]/bar: created\n")
    .expected_stderr(indoc! {"
      error: failed to create directory `[ROOT]/foo`: occupied
      error: 1 of 2 items failed
    "})
    .run()
}

#[test]
fn dry_run() -> Result {
  Test::new()?
    .config(indoc! {r#"
      clean = ["~"]
      create = ["~/qux", "~/waldo"]
      shell = ["echo garply"]

      [link]
      "~/bar/baz" = "foo"
      "~/quux" = "foo"
    "#})
    .file("foo", "")
    .symlink("quux", "corge")
    .symlink("waldo", "fred")
    .argument("--dry-run")
    .absent("bar")
    .absent("qux")
    .expected_link("quux", "corge")
    .expected_link("waldo", "fred")
    .expected_stdout(indoc! {"
      [ROOT]/quux: would remove
      [ROOT]/waldo: would remove
      [ROOT]/qux: would create
      [ROOT]/waldo: would create
      [ROOT]/bar/baz: would create
      [ROOT]/quux: would create
      echo garply: would run
    "})
    .run()
}

#[test]
fn dry_run_link_into_created_directory() -> Result {
  Test::new()?
    .config(indoc! {r#"
      create = ["~/foo"]

      [link]
      "~/foo" = "bar"
    "#})
    .file("bar", "")
    .argument("--dry-run")
    .absent("foo")
    .expected_status(1)
    .expected_stdout("[ROOT]/foo: would create\n")
    .expected_stderr(indoc! {"
      error: failed to link `[ROOT]/foo`: occupied
      error: 1 of 2 items failed
    "})
    .run()
}

#[test]
fn shell() -> Result {
  Test::new()?
    .directory("sub")
    .file("sub/strew.toml", r#"shell = ["pwd", ["echo foo", "bar"]]"#)
    .argument("--config")
    .argument("sub/strew.toml")
    .expected_stdout(indoc! {"
      pwd: running
      [ROOT]/sub
      bar: running
      foo
    "})
    .run()
}

#[test]
fn shell_failure() -> Result {
  Test::new()?
    .config(r#"shell = ["false", "echo foo"]"#)
    .expected_status(1)
    .expected_stdout(indoc! {"
      false: running
      echo foo: running
      foo
    "})
    .expected_stderr(indoc! {"
      error: failed to run `false`: exit status: 1
      error: 1 of 2 items failed
    "})
    .run()
}

#[test]
fn verbose() -> Result {
  Test::new()?
    .config(indoc! {r#"
      create = ["~/foo"]

      [link]
      "~/bar" = "baz"
    "#})
    .directory("foo")
    .file("baz", "")
    .symlink("bar", "baz")
    .argument("--verbose")
    .expected_stdout(indoc! {"
      [ROOT]/foo: exists
      [ROOT]/bar: linked
    "})
    .run()
}
