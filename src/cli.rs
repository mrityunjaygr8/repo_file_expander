use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rfe",
    color = clap::ColorChoice::Auto,
    subcommand_precedence_over_arg = true,
    dont_delimit_trailing_values = true,
    about = format!("woo woo")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn parse_and_resolve_options() -> Self {
        let cli = Self::parse();
        cli
    }
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    #[command(about = "Scaffold devnev.yaml, devenv.nix, .gitignore and .envrc")]
    Init {
        target: Option<PathBuf>,
        #[arg(short, long)]
        source: Option<String>,
    },
}
