use clap::Parser;
use flow::cli::{self, Options};

fn main() -> eyre::Result<()> {
    cli::run(Options::parse())
}
