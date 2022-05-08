use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use clap::Parser;
use color_eyre::eyre::{eyre, Context};
use color_eyre::{Help, Result};
use common::ShaderMetadata;
use harness::ConfigId;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;

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

    println!(
        "{:<id_width$} {} {}",
        "ID".if_supports_color(Stdout, |it| it.dimmed()),
        "|".if_supports_color(Stdout, |it| it.dimmed()),
        "Adapter Name".if_supports_color(Stdout, |it| it.dimmed()),
    );

    for _ in 0..id_width + 1 {
        print!("{}", "-".if_supports_color(Stdout, |it| it.dimmed()));
    }

    print!("{}", "+".if_supports_color(Stdout, |it| it.dimmed()));

    for _ in 0..name_width + 1 {
        print!("{}", "-".if_supports_color(Stdout, |it| it.dimmed()));
    }

    println!();

    for config in configs {
        let id = config.id;
        let name = config.adapter_name;
        println!(
            "{:<id_width$} {} {name}",
            id.if_supports_color(Stdout, |it| it.cyan()),
            "|".if_supports_color(Stdout, |it| it.dimmed()),
        );
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

    let executions = if options.configs.is_empty() {
        let configs = harness::default_configs();

        if configs.is_empty() {
            return Err(eyre!("failed to find any suitable default configurations")
                .with_note(|| "use the `list` command to see all available configurations"));
        }

        print!("no configurations specified, using defaults: ");

        for (index, config) in configs.iter().enumerate() {
            print!("{}", config.if_supports_color(Stdout, |it| it.cyan()));

            if index < configs.len() - 1 {
                print!(", ");
            }
        }

        println!();
        println!();

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
                println!(
                    "{}",
                    "mismatch".if_supports_color(Stdout, |text| text.red())
                );
                std::process::exit(1);
            } else {
                prev = execution;
            }
        }
    }

    println!("{}", "ok".if_supports_color(Stdout, |text| text.green()));

    Ok(())
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
