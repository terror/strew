use {
  arguments::Arguments,
  clap::Parser,
  config::Config,
  serde::{Deserialize, Serialize},
  state::State,
  std::{
    collections::HashMap,
    env,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, IsTerminal},
    path::{Path, PathBuf},
    process,
  },
  style::Style,
  subcommand::Subcommand,
};

mod arguments;
mod config;
mod state;
mod style;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
