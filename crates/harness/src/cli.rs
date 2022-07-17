use std::marker::PhantomData;
use std::time::Duration;

use clap::Parser;
use frontend::cli::RunOptions;
use frontend::ExecutionError;
use reflection::PipelineDescription;
use types::ConfigId;

use crate::{ExecutionEvent, ExecutionInput, ExecutionOutput, HarnessHost};

#[derive(Parser)]
pub enum Command {
    /// Lists available configurations that can be used to execute a shader.
    List,

    /// Runs a wgsl shader against one or more configurations.
    Run(RunOptions),

    #[clap(hide(true))]
    Exec {
        #[clap(action)]
        config: ConfigId,
    },

    /// Runs the harness server for remote execution.
    Serve(crate::server::Options),
}

pub fn run<Host: HarnessHost>(command: Command) -> eyre::Result<()> {
    match command {
        Command::List => list(),
        Command::Run(options) => execute::<Host>(options),
        Command::Exec { config } => internal_run(config),
        Command::Serve(options) => crate::server::run::<Host>(options),
    }
}

fn list() -> eyre::Result<()> {
    let frontend = frontend::Printer::new();
    frontend.print_all_configs(crate::query_configs())?;
    Ok(())
}

fn internal_run(config: ConfigId) -> eyre::Result<()> {
    let input: ExecutionInput =
        bincode::decode_from_std_read(&mut std::io::stdin(), bincode::config::standard())?;

    let output = ExecutionOutput {
        buffers: crate::execute_config(&input.shader, &input.pipeline_desc, &config)?,
    };

    bincode::encode_into_std_write(output, &mut std::io::stdout(), bincode::config::standard())?;

    Ok(())
}

pub fn execute<Host: HarnessHost>(options: RunOptions) -> eyre::Result<()> {
    struct Executor<Host>(PhantomData<Host>);

    impl<Host> Executor<Host> {
        fn new() -> Executor<Host> {
            Executor(PhantomData)
        }
    }

    impl<Host: HarnessHost> frontend::Executor for Executor<Host> {
        fn execute(
            &self,
            shader: &str,
            pipeline_desc: &PipelineDescription,
            configs: &[ConfigId],
            timeout: Option<Duration>,
            on_event: &mut dyn FnMut(ExecutionEvent) -> Result<(), ExecutionError>,
        ) -> Result<(), ExecutionError> {
            crate::execute::<Host, _>(shader, pipeline_desc, configs, timeout, on_event)
        }
    }

    frontend::cli::run(options, &Executor::<Host>::new())
}
