use {
  anyhow::{Context, bail},
  arguments::Arguments,
  clap::Parser,
  config::Config,
  link::Status,
  serde::Deserialize,
  std::{
    collections::BTreeMap,
    env,
    fmt::{self, Display, Formatter},
    fs, io,
    path::{Path, PathBuf},
    process,
  },
  subcommand::Subcommand,
};

#[cfg(test)]
use {std::os::unix::fs::symlink, tempfile::TempDir};

mod arguments;
mod config;
mod link;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error:#}");
    process::exit(1);
  }
}
