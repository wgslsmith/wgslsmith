use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use clap::Parser;
use color_eyre::Help;
use eyre::{eyre, Context};
use frontend::ExecutionResult;
use reflection::ResourceKind;
use types::ConfigId;

use crate::utils;

#[derive(Parser)]
pub enum Command {
    /// Lists available configurations that can be used to execute a shader.
    List,

    /// Runs a wgsl shader against one or more configurations.
    Run(RunOptions),

    /// Runs the harness server for remote execution.
    Serve(crate::server::Options),
}

pub fn run(command: Command) -> eyre::Result<()> {
    match command {
        Command::List => list(),
        Command::Run(options) => execute(options),
        Command::Serve(options) => crate::server::run(options),
    }
}

pub fn list() -> eyre::Result<()> {
    frontend::print_all_configs(crate::query_configs())
}

#[derive(Parser)]
pub struct RunOptions {
    /// Path to wgsl shader program to be executed (use '-' for stdin)
    #[clap(action, default_value = "-")]
    input: String,

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

pub fn execute(options: RunOptions) -> eyre::Result<()> {
    let shader = read_shader_from_path(&options.input)?;

    let mut input_data = read_input_data(&options)?;

    let module = parser::parse(&shader);
    let mut pipeline_desc = reflection::reflect(&module, |resource| {
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

    let executions = if options.configs.is_empty() {
        let configs = crate::default_configs();

        if configs.is_empty() {
            return Err(eyre!("failed to find any suitable default configurations")
                .with_note(|| "use the `list` command to see all available configurations"));
        }

        frontend::print_default_configs(&configs)?;

        crate::execute(&shader, &pipeline_desc, &configs)?
    } else {
        crate::execute(&shader, &pipeline_desc, &options.configs)?
    };

    if executions.is_empty() {
        return Ok(());
    }

    let mut executions = executions.into_iter();

    if let Some(mut prev) = executions.next() {
        for execution in executions {
            for (i, resource) in pipeline_desc
                .resources
                .iter()
                .filter(|it| it.kind == ResourceKind::StorageBuffer)
                .enumerate()
            {
                for (offset, size) in resource.type_desc.ranges() {
                    let range = offset..(offset + size);
                    if execution.results[i][range.clone()] != prev.results[i][range] {
                        frontend::print_execution_result(ExecutionResult::Mismatch)?;
                        std::process::exit(1);
                    }
                }
            }

            prev = execution;
        }
    }

    frontend::print_execution_result(ExecutionResult::Ok)?;

    Ok(())
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
            if options.input != "-" {
                if let Some(path) = Path::new(&options.input)
                    .parent()
                    .map(|it| it.join("inputs.json"))
                {
                    if path.exists() {
                        return Ok(serde_json::from_reader(File::open(path)?)?);
                    }
                }

                let path = Path::new(&options.input).with_extension("json");
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
