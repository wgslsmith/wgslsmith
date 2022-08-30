use clap::Parser;
use ub::cli::{self, Options};

fn main() -> eyre::Result<()> {
    cli::run(Options::parse())
}
