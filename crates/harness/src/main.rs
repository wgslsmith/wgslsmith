use clap::Parser;
use harness::cli::{self, Command};
use harness::HarnessHost;

fn main() -> eyre::Result<()> {
    if std::env::var("NO_COLOR") == Err(std::env::VarError::NotPresent) {
        color_eyre::install()?;
    } else {
        color_eyre::config::HookBuilder::new()
            .theme(color_eyre::config::Theme::new())
            .install()?;
    }

    env_logger::init();

    struct Host;

    impl HarnessHost for Host {
        fn exec_command() -> std::process::Command {
            let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
            cmd.arg("exec");
            cmd
        }
    }

    cli::run::<Host>(Command::parse())
}
