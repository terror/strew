use {
  action::Action,
  anyhow::{Context, bail},
  arguments::Arguments,
  clap::Parser,
  config::{Config, Shell},
  link::Status,
  plan::Plan,
  serde::Deserialize,
  std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::{self, Display, Formatter},
    fs, io,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{self, Command},
  },
  subcommand::Subcommand,
};

#[cfg(test)]
use tempfile::TempDir;

mod action;
mod apply;
mod arguments;
mod config;
mod directory;
mod link;
mod plan;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error:#}");
    process::exit(1);
  }
}
