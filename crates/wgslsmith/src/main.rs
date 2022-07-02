#[cfg(all(target_family = "unix", feature = "reducer"))]
mod compiler;
mod config;
mod fmt;
mod fuzzer;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod reducer;
mod remote;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod test;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod validator;

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use eyre::Context;

#[derive(Parser)]
struct Options {
    #[clap(long, action)]
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
    /// Execute a shader.
    #[cfg(feature = "harness")]
    Run(harness::cli::RunOptions),
    #[cfg(feature = "harness")]
    Harness {
        #[clap(subcommand)]
        cmd: harness::cli::Command,
    },
    /// Interact with a remote harness server.
    Remote {
        #[clap(subcommand)]
        cmd: RemoteCmd,
        #[clap(action)]
        server: String,
    },
}

#[derive(Parser)]
enum RemoteCmd {
    List,
}

fn main() -> eyre::Result<()> {
    if std::env::var("NO_COLOR") == Err(std::env::VarError::NotPresent) {
        color_eyre::install()?;
    } else {
        color_eyre::config::HookBuilder::new()
            .theme(color_eyre::config::Theme::new())
            .install()?;
    }

    let options = Options::parse();

    let config_file = options
        .config_file
        .ok_or(())
        .or_else(|_| config::default_path())
        .wrap_err("couldn't determine config file path")?;

    let config = config::Config::load(&config_file)?;

    match options.cmd {
        Cmd::Config => {
            if let Some(dir) = config_file.parent() {
                fs::create_dir_all(dir)?;
            }
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
        #[cfg(feature = "harness")]
        Cmd::Run(options) => harness::cli::execute::<HarnessHost>(options),
        #[cfg(feature = "harness")]
        Cmd::Harness { cmd } => harness::cli::run::<HarnessHost>(cmd),
        Cmd::Remote { cmd, server } => match cmd {
            RemoteCmd::List => {
                let address = config.resolve_remote(&server);
                let res = remote::query_configs(address)?;
                harness_frontend::Printer::new().print_all_configs(res.configs)
            }
        },
    }
}

#[cfg(feature = "harness")]
struct HarnessHost;

#[cfg(feature = "harness")]
impl harness::HarnessHost for HarnessHost {
    fn exec_command() -> std::process::Command {
        let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
        cmd.args(["harness", "exec"]);
        cmd
    }
}
