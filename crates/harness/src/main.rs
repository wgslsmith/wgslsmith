mod server;

use clap::Parser;

#[derive(Parser)]
enum Command {
    /// Lists available configurations that can be used to execute a shader.
    List,

    /// Runs a wgsl shader against one or more configurations.
    Run(harness::cli::RunOptions),

    /// Runs the harness server for remote execution.
    Serve(server::Options),
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

    match Command::parse() {
        Command::List => harness::cli::list(),
        Command::Run(options) => harness::cli::run(options),
        Command::Serve(options) => server::run(options),
    }
}
