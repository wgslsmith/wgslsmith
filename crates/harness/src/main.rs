use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

use clap::Parser;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use common::ShaderMetadata;

#[derive(Parser)]
struct Options {
    /// Path to wgsl shader program to be executed (use '-' for stdin)
    #[clap(default_value_t = String::from("-"))]
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
    #[clap(long)]
    metadata: Option<String>,
}

fn main() -> Result<()> {
    let options = Options::parse();

    color_eyre::install()?;
    env_logger::init();

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

    let execution = harness::execute(&shader, &meta)?;

    println!("========== Results ==========");
    println!("dawn: result={:x?}", execution.dawn);
    println!("wgpu: result={:x?}", execution.wgpu);

    if execution.dawn != execution.wgpu {
        println!("mismatch!");
        std::process::exit(1);
    }

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
