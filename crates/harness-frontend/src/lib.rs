mod printer;
mod utils;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::{fmt, io};

use eyre::{eyre, Context};
use reflection::PipelineDescription;

pub use printer::{ExecutionEvent, ExecutionResult, Printer};
use types::ConfigId;

pub fn read_input_data(
    shader: &str,
    input_data: Option<&str>,
) -> eyre::Result<HashMap<String, Vec<u8>>> {
    match input_data {
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
                    Err(e) if e.kind() == io::ErrorKind::NotFound => Err(parse_err),
                    // Found file but failed to open it
                    Err(e) => Err(e.into()),
                },
            }
        }
        None => {
            // Don't look for file if shader was passed over stdin
            if shader != "-" {
                if let Some(path) = Path::new(shader).parent().map(|it| it.join("inputs.json")) {
                    if path.exists() {
                        return Ok(serde_json::from_reader(File::open(path)?)?);
                    }
                }

                let path = Path::new(shader).with_extension("json");
                if path.exists() {
                    return Ok(serde_json::from_reader(File::open(path)?)?);
                }
            }

            // Default to no input data
            Ok(Default::default())
        }
    }
}

pub fn read_shader_from_path(path: &str) -> eyre::Result<String> {
    let mut input: Box<dyn io::Read> = match path {
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

pub fn reflect_shader(
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

#[derive(Debug)]
pub enum ExecutionError {
    NoDefaultConfigs,
    Io(io::Error),
    Encode(bincode::error::EncodeError),
    Decode(bincode::error::DecodeError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::NoDefaultConfigs => write!(f, "no suitable default configs found"),
            ExecutionError::Io(e) => e.fmt(f),
            ExecutionError::Encode(e) => e.fmt(f),
            ExecutionError::Decode(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<io::Error> for ExecutionError {
    fn from(e: io::Error) -> Self {
        ExecutionError::Io(e)
    }
}

impl From<bincode::error::EncodeError> for ExecutionError {
    fn from(e: bincode::error::EncodeError) -> Self {
        ExecutionError::Encode(e)
    }
}

impl From<bincode::error::DecodeError> for ExecutionError {
    fn from(e: bincode::error::DecodeError) -> Self {
        ExecutionError::Decode(e)
    }
}

pub trait Executor {
    fn execute(
        &self,
        shader: &str,
        pipeline_desc: &PipelineDescription,
        configs: &[ConfigId],
        on_event: &mut dyn FnMut(ExecutionEvent) -> Result<(), ExecutionError>,
    ) -> Result<(), ExecutionError>;
}

pub mod cli {
    use clap::Parser;
    use color_eyre::Help;
    use eyre::eyre;
    use types::ConfigId;

    use crate::{ExecutionEvent, ExecutionResult, Executor};

    #[derive(Parser)]
    pub struct RunOptions {
        /// Path to wgsl shader program to be executed (use '-' for stdin)
        #[clap(action, default_value = "-")]
        pub shader: String,

        /// Input data for uniform buffers.
        #[clap(action)]
        pub input_data: Option<String>,

        /// List of configurations to test.
        ///
        /// Configurations must be specified using their IDs. Use the `list` command to see available
        /// configurations.
        ///
        /// If no configurations are provided, defaults will be selected for this platform.
        #[clap(short, long = "config", action)]
        pub configs: Vec<ConfigId>,
    }

    pub fn run(options: RunOptions, executor: &dyn Executor) -> eyre::Result<()> {
        let shader = super::read_shader_from_path(&options.shader)?;
        let input_data = super::read_input_data(&options.shader, options.input_data.as_deref())?;
        let (pipeline_desc, type_descs) = super::reflect_shader(&shader, input_data);

        let printer = super::Printer::new();

        let mut executions = vec![];
        let mut is_fail = false;
        let mut on_event = |event: ExecutionEvent| {
            printer.print_execution_event(&event, &pipeline_desc)?;
            if let ExecutionEvent::Success(buffers) = event {
                executions.push(buffers);
            } else if let ExecutionEvent::Failure(_) = event {
                is_fail = true
            }
            Ok(())
        };

        executor
            .execute(&shader, &pipeline_desc, &options.configs, &mut on_event)
            .map_err(|e| match e {
                crate::ExecutionError::NoDefaultConfigs => {
                    eyre!("failed to find any suitable default configurations")
                        .with_note(|| "use the `list` command to see all available configurations")
                }
                e => eyre!(e),
            })?;

        if is_fail {
            panic!("one or more executions failed");
        }

        if buffer_check::compare(executions.iter(), &pipeline_desc, &type_descs) {
            printer.print_execution_result(ExecutionResult::Ok)?;
        } else {
            printer.print_execution_result(ExecutionResult::Mismatch)?;
            std::process::exit(1);
        }

        Ok(())
    }
}
