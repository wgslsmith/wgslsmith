use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use clap::Parser;
use color_eyre::eyre::{eyre, Context};
use color_eyre::{Help, Result};
use common::ShaderMetadata;
use harness::ConfigId;
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Parser)]
struct RunOptions {
    /// Path to wgsl shader program to be executed (use '-' for stdin)
    #[clap(default_value = "-")]
    input: String,

    /// Shader metadata, describing inputs and outputs.
    ///
    /// This can be a path to a JSON file, or an inline JSON string.
    /// The format is described by the ShaderMetadata structure at
    /// https://github.com/wgslsmith/wgslsmith/blob/main/crates/common/src/lib.rs.
    ///
    /// If no value is supplied, we look for a JSON file with the same name and parent directory
    /// as the shader file.
    /// Failing that, it will be assumed that the shader has no I/O.
    metadata: Option<String>,

    /// List of configurations to test.
    ///
    /// Configurations must be specified using their IDs. Use the `list` command to see available
    /// configurations.
    ///
    /// If no configurations are provided, a set of platform-specific defaults will be used.
    #[clap(short, long = "config")]
    configs: Vec<ConfigId>,
}

#[derive(Parser)]
enum Command {
    /// Lists available configurations that can be used to execute a shader.
    List,

    /// Runs a wgsl shader against one or more configurations.
    Run(RunOptions),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    match Command::parse() {
        Command::List => list(),
        Command::Run(options) => exec(options),
    }
}

fn list() -> Result<()> {
    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);

    let configs = harness::Config::all();

    let id_width = configs
        .iter()
        .map(|it| it.id.to_string().len())
        .max()
        .unwrap_or(0);

    let name_width = configs
        .iter()
        .map(|it| it.adapter_name.len())
        .max()
        .unwrap_or(0);

    stdout.set_color(&dimmed())?;

    writeln!(&mut stdout, "{:<id_width$} | Adapter Name", "ID")?;

    for _ in 0..id_width + 1 {
        write!(&mut stdout, "-")?;
    }

    write!(&mut stdout, "+")?;

    for _ in 0..name_width + 1 {
        write!(&mut stdout, "-")?;
    }

    stdout.reset()?;
    writeln!(&mut stdout)?;

    for config in configs {
        let id = config.id;
        let name = config.adapter_name;

        stdout.set_color(&cyan())?;
        write!(&mut stdout, "{id:<id_width$}")?;

        stdout.set_color(&dimmed())?;
        write!(&mut stdout, " | ")?;

        stdout.reset()?;
        writeln!(&mut stdout, "{name}")?;
    }

    Ok(())
}

fn exec(options: RunOptions) -> Result<()> {
    let shader = read_shader_from_path(&options.input)?;

    let meta = match options.metadata.as_deref() {
        Some(meta) => {
            // Try parsing value as json string
            match serde_json::from_str(meta)
                .wrap_err_with(|| eyre!("failed to parse shader metadata"))
            {
                Ok(meta) => meta,
                // On failure, try treating value as file path
                Err(parse_err) => match File::open(meta) {
                    // File opened successfully, parse the contents as json
                    Ok(file) => serde_json::from_reader(file)
                        .wrap_err_with(|| eyre!("failed to parse shader metadata"))?,
                    // File not found, return original parsing error
                    Err(e) if e.kind() == ErrorKind::NotFound => return Err(parse_err),
                    // Found file but failed to open it
                    Err(e) => return Err(e.into()),
                },
            }
        }
        None => {
            let mut meta = None;

            // Don't look for metadata file if shader was passed over stdin
            if options.input != "-" {
                match File::open(Path::new(&options.input).with_extension("json")) {
                    // Found a metadata file next to the shader file
                    Ok(file) => meta = Some(serde_json::from_reader(&file)?),
                    // Found metadata file but failed to open it
                    Err(e) if e.kind() != ErrorKind::NotFound => return Err(e.into()),
                    // Failed to find metadata file
                    _ => {}
                };
            }

            // Default to empty resource list
            meta.unwrap_or_else(|| ShaderMetadata { resources: vec![] })
        }
    };

    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);

    let executions = if options.configs.is_empty() {
        let configs = harness::default_configs();

        if configs.is_empty() {
            return Err(eyre!("failed to find any suitable default configurations")
                .with_note(|| "use the `list` command to see all available configurations"));
        }

        write!(&mut stdout, "no configurations specified, using defaults: ")?;

        for (index, config) in configs.iter().enumerate() {
            stdout.set_color(&cyan())?;
            write!(&mut stdout, "{config}")?;
            stdout.reset()?;

            if index < configs.len() - 1 {
                write!(&mut stdout, ", ")?;
            }
        }

        writeln!(&mut stdout)?;
        writeln!(&mut stdout)?;

        harness::execute(&shader, &meta, &configs)?
    } else {
        harness::execute(&shader, &meta, &options.configs)?
    };

    if executions.is_empty() {
        return Ok(());
    }

    let mut executions = executions.into_iter();

    if let Some(mut prev) = executions.next() {
        for execution in executions {
            if prev.results != execution.results {
                stdout.set_color(&red())?;
                writeln!(&mut stdout, "mismatch")?;
                stdout.reset()?;
                std::process::exit(1);
            } else {
                prev = execution;
            }
        }
    }

    stdout.set_color(&green())?;
    writeln!(&mut stdout, "ok")?;
    stdout.reset()?;

    Ok(())
}

fn dimmed() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_dimmed(true);
    spec
}

fn cyan() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Cyan));
    spec
}

fn red() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Red));
    spec
}

fn green() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Green));
    spec
}

fn read_shader_from_path(path: &str) -> Result<String> {
    let mut input: Box<dyn Read> = match path {
        "-" => Box::new(std::io::stdin()),
        path => Box::new(File::open(path)?),
    };

    let mut shader = String::new();
    input.read_to_string(&mut shader)?;

    Ok(shader)
}
