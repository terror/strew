use {
  arguments::Arguments,
  clap::Parser,
  config::Config,
  link::Link,
  link_state::LinkState,
  serde::{Deserialize, Serialize},
  std::{
    collections::BTreeMap,
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
mod link;
mod link_state;
mod style;
mod subcommand;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
