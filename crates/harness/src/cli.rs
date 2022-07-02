use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use clap::Parser;
use color_eyre::Help;
use eyre::{eyre, Context};
use frontend::ExecutionResult;
use reflection::{PipelineDescription, ResourceKind};
use types::ConfigId;

use crate::{utils, ExecutionEvent, ExecutionInput, ExecutionOutput, HarnessHost};

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
        Command::Serve(options) => crate::server::run(options),
    }
}

pub fn list() -> eyre::Result<()> {
    frontend::Printer::new().print_all_configs(crate::query_configs())
}

#[derive(Parser)]
pub struct RunOptions {
    /// Path to wgsl shader program to be executed (use '-' for stdin)
    #[clap(action, default_value = "-")]
    shader: String,

    /// Input data for uniform buffers.
    #[clap(action)]
    input_data: Option<String>,

    /// List of configurations to test.
    ///
    /// Configurations must be specified using their IDs. Use the `list` command to see available
    /// configurations.
    ///
    /// If no configurations are provided, defaults will be selected for this platform.
    #[clap(short, long = "config", action)]
    configs: Vec<ConfigId>,
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
    let shader = read_shader_from_path(&options.shader)?;

    let input_data = read_input_data(&options)?;
    let (pipeline_desc, type_descs) = reflect_shader(&shader, input_data);

    let frontend = frontend::Printer::new();

    let mut configs = options.configs;
    if configs.is_empty() {
        configs = crate::default_configs();

        if configs.is_empty() {
            return Err(eyre!("failed to find any suitable default configurations")
                .with_note(|| "use the `list` command to see all available configurations"));
        }

        frontend.print_default_configs(&configs)?;
    }

    pub struct Execution {
        pub config: ConfigId,
        pub results: Vec<Vec<u8>>,
    }

    let mut current_config = None;
    let mut executions = vec![];

    crate::execute::<Host, _>(
        &shader,
        &pipeline_desc,
        &configs,
        |event: ExecutionEvent| {
            frontend.print_execution_event(&event, &pipeline_desc)?;
            match event {
                ExecutionEvent::Start(config) => current_config = Some(config),
                ExecutionEvent::Success(buffers) => executions.push(Execution {
                    config: current_config.take().unwrap(),
                    results: buffers,
                }),
                ExecutionEvent::Failure(_) => current_config = None,
            }
            Ok(())
        },
    )?;

    if executions.len() != configs.len() {
        panic!("one or more executions failed");
    }

    if executions.is_empty() {
        return Ok(());
    }

    let mut executions = executions.into_iter();

    if let Some(mut prev) = executions.next() {
        for execution in executions {
            for (i, _) in pipeline_desc
                .resources
                .iter()
                .filter(|it| it.kind == ResourceKind::StorageBuffer)
                .enumerate()
            {
                for (offset, size) in type_descs[i].ranges() {
                    let range = offset..(offset + size);
                    if execution.results[i][range.clone()] != prev.results[i][range] {
                        frontend.print_execution_result(ExecutionResult::Mismatch)?;
                        std::process::exit(1);
                    }
                }
            }

            prev = execution;
        }
    }

    frontend.print_execution_result(ExecutionResult::Ok)?;

    Ok(())
}

fn reflect_shader(
    shader: &str,
    mut input_data: HashMap<String, Vec<u8>>,
) -> (PipelineDescription, Vec<common::Type>) {
    let module = parser::parse(shader);

    let (mut pipeline_desc, type_descs) = reflection::reflect(&module, |resource| {
        input_data.remove(&format!("{}:{}", resource.group, resource.binding))
    });

    let mut resource_vars = HashSet::new();

    for resource in &pipeline_desc.resources {
        resource_vars.insert(resource.name.clone());
    }

    utils::remove_accessed_vars(&mut resource_vars, &module);

    pipeline_desc
        .resources
        .retain(|resource| !resource_vars.contains(&resource.name));

    (pipeline_desc, type_descs)
}

fn read_input_data(options: &RunOptions) -> eyre::Result<HashMap<String, Vec<u8>>> {
    match options.input_data.as_deref() {
        Some(input_data) => {
            // Try parsing value as json string
            match serde_json::from_str(input_data)
                .wrap_err_with(|| eyre!("failed to parse input data"))
            {
                Ok(input_data) => Ok(input_data),
                // On failure, try treating value as file path
                Err(parse_err) => match File::open(input_data) {
                    // File opened successfully, parse the contents as json
                    Ok(file) => serde_json::from_reader(file)
                        .wrap_err_with(|| eyre!("failed to parse input data")),
                    // File not found, return original parsing error
                    Err(e) if e.kind() == ErrorKind::NotFound => Err(parse_err),
                    // Found file but failed to open it
                    Err(e) => Err(e.into()),
                },
            }
        }
        None => {
            // Don't look for file if shader was passed over stdin
            if options.shader != "-" {
                if let Some(path) = Path::new(&options.shader)
                    .parent()
                    .map(|it| it.join("inputs.json"))
                {
                    if path.exists() {
                        return Ok(serde_json::from_reader(File::open(path)?)?);
                    }
                }

                let path = Path::new(&options.shader).with_extension("json");
                if path.exists() {
                    return Ok(serde_json::from_reader(File::open(path)?)?);
                }
            }

            // Default to no input data
            Ok(Default::default())
        }
    }
}

fn read_shader_from_path(path: &str) -> eyre::Result<String> {
    let mut input: Box<dyn Read> = match path {
        "-" => Box::new(std::io::stdin()),
        path => {
            Box::new(File::open(path).wrap_err_with(|| eyre!("Failed to open file at '{path}'"))?)
        }
    };

    let mut shader = String::new();
    input
        .read_to_string(&mut shader)
        .wrap_err_with(|| eyre!("Failed to read shader from '{path}'"))?;

    Ok(shader)
}
