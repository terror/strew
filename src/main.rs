use {
  action::Action,
  anstyle::{AnsiColor, Style},
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

fn print_error(error: &anyhow::Error) {
  let style = AnsiColor::Red.on_default().bold();
  anstream::eprintln!("{style}error{style:#}: {error:#}");
}

fn main() {
  if let Err(error) = Arguments::parse().run() {
    print_error(&error);
    process::exit(1);
  }
}
