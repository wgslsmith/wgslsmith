#[cfg(all(target_family = "unix", feature = "reducer"))]
mod compiler;
mod config;
mod executor;
mod fmt;
mod fuzzer;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod reducer;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod test;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod validator;

use std::fs;
use std::path::PathBuf;

use clap::{ArgEnum, Parser};
use directories::ProjectDirs;

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
    /// Generate a random shader.
    Gen(generator::Options),
    /// Recondition a shader to add safety checks.
    Recondition(reconditioner::cli::Options),
    /// Format a shader.
    Fmt(fmt::Options),
    Fuzz(fuzzer::Options),
    /// Reduce a shader.
    #[cfg(all(target_family = "unix", feature = "reducer"))]
    Reduce(reducer::Options),
    #[cfg(all(target_family = "unix", feature = "reducer"))]
    Test(test::Options),
    // Exec(executor::Options),
    /// Execute a shader.
    #[cfg(feature = "harness")]
    Run(harness::cli::RunOptions),
    List {
        #[clap(arg_enum)]
        kind: ObjectKind,
    },
}

#[derive(ArgEnum, Clone)]
enum ObjectKind {
    #[cfg(feature = "harness")]
    Executors,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let options = Options::parse();

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

    match options.cmd {
        Cmd::Config => {
            fs::create_dir_all(&config_dir)?;
            edit::edit_file(&config_file)?;
            Ok(())
        }
        Cmd::Gen(options) => generator::run(options),
        Cmd::Recondition(options) => reconditioner::cli::run(options),
        Cmd::Fmt(options) => fmt::run(options),
        Cmd::Fuzz(options) => fuzzer::run(config, options),
        #[cfg(all(target_family = "unix", feature = "reducer"))]
        Cmd::Reduce(options) => reducer::run(config, options),
        #[cfg(all(target_family = "unix", feature = "reducer"))]
        Cmd::Test(options) => test::run(&config, options),
        // Cmd::Exec(options) => executor::run(options),
        #[cfg(feature = "harness")]
        Cmd::Run(options) => harness::cli::run(options),
        Cmd::List { kind } => match kind {
            #[cfg(feature = "harness")]
            ObjectKind::Executors => harness::cli::list(),
        },
    }
}
