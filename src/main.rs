use {
  anyhow::Context,
  arguments::Arguments,
  clap::Parser,
  config::Config,
  serde::Deserialize,
  std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process,
  },
};

mod arguments;
mod config;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error:#}");
    process::exit(1);
  }
}
