use {
  anyhow::Context,
  arguments::Arguments,
  clap::Parser,
  config::Config,
  serde::{Deserialize, Serialize},
  state::State,
  std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
  },
  subcommand::Subcommand,
};

mod arguments;
mod config;
mod state;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
