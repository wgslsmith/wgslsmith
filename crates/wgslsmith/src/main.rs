mod executor;
mod fuzzer;
mod gen;
mod reconditioner;
mod reducer;
mod test;

use std::env;
use std::path::PathBuf;

use clap::Parser;
use eyre::eyre;
use tap::Pipe;

#[derive(Parser)]
enum Cmd {
    Gen(gen::Options),
    Recondition(reconditioner::Options),
    Fuzz(fuzzer::Options),
    Reduce(reducer::Options),
    Test,
    Exec(executor::Options),
    #[clap(disable_help_flag(true), allow_hyphen_values(true))]
    Harness {
        args: Vec<String>,
    },
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let root = PathBuf::from(env::var("WGSLSMITH_ROOT").unwrap());
    let harness_target = env::var("HARNESS_TARGET").ok();

    let mut harness_path = root
        .join("harness/target")
        .pipe(|it| {
            if let Some(target) = &harness_target {
                it.join(target)
            } else {
                it
            }
        })
        .join("release/harness");

    if matches!(harness_target, Some(target) if target.contains("windows")) {
        harness_path.set_extension("exe");
    }

    match Cmd::parse() {
        Cmd::Gen(options) => gen::run(options),
        Cmd::Recondition(options) => reconditioner::run(options),
        Cmd::Fuzz(options) => fuzzer::run(options),
        Cmd::Reduce(options) => reducer::run(options),
        Cmd::Test => test::run(),
        Cmd::Exec(options) => executor::run(options),
        Cmd::Harness { args } => {
            let status = std::process::Command::new(harness_path)
                .args(args)
                .status()?
                .code()
                .ok_or_else(|| eyre!("missing status code"))?;
            std::process::exit(status);
        }
    }
}
