mod compiler;
mod config;
mod executor;
mod fmt;
mod fuzzer;
mod reconditioner;
mod reducer;
mod test;
mod validator;

use std::path::PathBuf;
use std::{env, fs};

use clap::Parser;
use directories::ProjectDirs;
use eyre::{eyre, Context};
use tap::Pipe;

#[derive(Parser)]
struct Options {
    #[clap(long)]
    config_file: Option<PathBuf>,
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Parser)]
enum Cmd {
    /// Open the wgslsmith config file in the default text editor.
    Config,
    Gen(generator::Options),
    Recondition(reconditioner::Options),
    Fmt(fmt::Options),
    Fuzz(fuzzer::Options),
    Reduce(reducer::Options),
    Test(test::Options),
    Exec(executor::Options),
    #[clap(disable_help_flag(true), allow_hyphen_values(true))]
    Harness {
        args: Vec<String>,
    },
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let options = Options::parse();

    let root = PathBuf::from(env::var("WGSLSMITH_ROOT").unwrap());

    let exe = std::env::current_exe()?;
    let project_dirs = ProjectDirs::from("", "", "wgslsmith");
    let config_dir = if let Some(dirs) = &project_dirs {
        dirs.config_dir()
    } else {
        exe.parent().unwrap()
    };

    let config_file = options
        .config_file
        .unwrap_or_else(|| config_dir.join("wgslsmith.toml"));

    let config = config::Config::load(&config_file)?;

    let mut harness_path = root
        .join("target")
        .pipe(|it| {
            if let Some(target) = &config.harness.target {
                it.join(target)
            } else {
                it
            }
        })
        .join("release/wgslsmith-harness");

    if matches!(&config.harness.target, Some(target) if target.contains("windows")) {
        harness_path.set_extension("exe");
    }

    match options.cmd {
        Cmd::Config => {
            fs::create_dir_all(&config_dir)?;
            edit::edit_file(&config_file)?;
            Ok(())
        }
        Cmd::Gen(options) => generator::run(options),
        Cmd::Recondition(options) => reconditioner::run(options),
        Cmd::Fmt(options) => fmt::run(options),
        Cmd::Fuzz(options) => fuzzer::run(config, options),
        Cmd::Reduce(options) => reducer::run(config, options),
        Cmd::Test(options) => test::run(&config, options),
        Cmd::Exec(options) => executor::run(options),
        Cmd::Harness { args } => {
            let status = std::process::Command::new(&harness_path)
                .args(args)
                .status()
                .wrap_err_with(|| eyre!("failed to execute `{}`", harness_path.display()))?
                .code()
                .ok_or_else(|| eyre!("missing status code"))?;
            std::process::exit(status);
        }
    }
}
