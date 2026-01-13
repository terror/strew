use {
  anyhow::Context,
  config::Config,
  serde::{Deserialize, Serialize},
  std::{collections::HashMap, process},
};

mod config;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

fn run() -> Result {
  let config = Config::load()?;

  println!("Loaded {} file entries:", config.files.len());

  for (name, entry) in &config.files {
    println!("  {}: {} -> {}", name, entry.source, entry.target);
  }

  Ok(())
}

fn main() {
  if let Err(error) = run() {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
