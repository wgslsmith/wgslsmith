#[cfg(all(target_family = "unix", feature = "reducer"))]
mod compiler;
mod config;
mod fmt;
mod fuzzer;
mod harness_runner;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod reducer;
mod remote;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod test;
#[cfg(all(target_family = "unix", feature = "reducer"))]
mod validator;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use color_eyre::Help;
use eyre::{eyre, Context};
use harness_frontend::{ExecutionError, ExecutionEvent};
use harness_types::ConfigId;
use reflection_types::PipelineDescription;

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
    /// Generate a random shader with data race
    DataRaceGen(data_race_generator::Options),
    /// Recondition a shader to add safety checks.
    Recondition(reconditioner::cli::Options),
    /// Add Flow Analysis to a shader.
    Flow(flow::cli::Options),
    /// Insert Undefined Behavour into a shader.
    UB(ub::cli::Options),
    /// Make it parallel!
    Thread(thread::cli::Options),
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
    Run(harness_frontend::cli::RunOptions),
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
        server: Option<String>,
    },
}

#[derive(Parser)]
enum RemoteCmd {
    List,
    Run(harness_frontend::cli::RunOptions),
}

fn main() -> eyre::Result<()> {
    if std::env::var("NO_COLOR") == Err(std::env::VarError::NotPresent) {
        color_eyre::install()?;
    } else {
        color_eyre::config::HookBuilder::new()
            .theme(color_eyre::config::Theme::new())
            .install()?;
    }

    env_logger::init();

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
        Cmd::DataRaceGen(options) => data_race_generator::run(options),
        Cmd::Recondition(options) => reconditioner::cli::run(options),
        Cmd::Flow(options) => flow::cli::run(options),
        Cmd::UB(options) => ub::cli::run(options),
        Cmd::Thread(options) => thread::cli::run(options),
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
        Cmd::Remote { cmd, server } => {
            let address = server
                .as_deref()
                .map(|server| config.resolve_remote(server))
                .or_else(|| config.default_remote())
                .ok_or_else(|| {
                    eyre!("no remote specified and no default remote found in config")
                        .with_note(|| "specify a default remote using the `harness.remote` field in your config file")
                })?;

            match cmd {
                RemoteCmd::List => {
                    let res = remote::list(address)?;
                    harness_frontend::Printer::new().print_all_configs(res.configs)?;
                    Ok(())
                }
                RemoteCmd::Run(options) => {
                    struct Executor<'a>(&'a str);

                    impl harness_frontend::Executor for Executor<'_> {
                        fn execute(
                            &self,
                            shader: &str,
                            workgroups: u32,
                            flow: bool,
                            pipeline_desc: &PipelineDescription,
                            configs: &[ConfigId],
                            timeout: Option<Duration>,
                            on_event: &mut dyn FnMut(ExecutionEvent) -> Result<(), ExecutionError>,
                        ) -> Result<(), ExecutionError> {
                            remote::execute(
                                self.0,
                                shader.to_owned(),
                                workgroups,
                                flow,
                                pipeline_desc.clone(),
                                configs.to_owned(),
                                timeout,
                                on_event,
                            )
                        }
                    }

                    harness_frontend::cli::run(options, &Executor(address))
                }
            }
        }
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
