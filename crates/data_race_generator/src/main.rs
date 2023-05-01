use clap::StructOpt;
use data_race_generator::Options;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    data_race_generator::run(Options::parse())
}
