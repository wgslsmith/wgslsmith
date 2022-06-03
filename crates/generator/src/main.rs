use clap::StructOpt;
use generator::Options;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    generator::run(Options::parse())
}
