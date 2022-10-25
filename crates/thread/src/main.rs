use clap::Parser;
use thread::cli::{self, Options};

fn main() -> eyre::Result<()> {
    cli::run(Options::parse())
}
