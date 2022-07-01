use clap::Parser;
use reconditioner::cli::{self, Options};

fn main() -> eyre::Result<()> {
    cli::run(Options::parse())
}
