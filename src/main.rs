use {
  anyhow::{Context, ensure},
  arguments::Arguments,
  clap::Parser,
  config::Config,
  serde::{Deserialize, Serialize},
  std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
  },
  subcommand::Subcommand,
};

#[cfg(unix)]
use std::os::unix;

#[cfg(windows)]
use std::os::windows;

mod arguments;
mod config;
mod linker;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
