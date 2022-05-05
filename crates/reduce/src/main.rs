use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use clap::Parser;
use which::Error;

#[derive(Parser)]
struct Options {
    /// Path to the WGSL shader file to reduce.
    shader: PathBuf,

    /// Path to the JSON metadata file.
    ///
    /// If not set, the program will look for a JSON file with the same name as the shader.
    metadata: Option<PathBuf>,

    /// Address of harness server.
    #[clap(short, long, default_value = "localhost:8080")]
    server: String,
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let shader_path = Path::new(&options.shader);
    if !shader_path.exists() {
        return Err(anyhow!("shader at {shader_path:?} does not exist"));
    }

    let shader_path = shader_path.canonicalize()?;

    let metadata_path = if let Some(metadata_path) = options.metadata {
        metadata_path
    } else {
        shader_path
            .parent()
            .unwrap()
            .join(shader_path.file_stem().unwrap())
            .with_extension("json")
    };

    if !metadata_path.exists() {
        return Err(anyhow!("metadata file at {metadata_path:?} does not exist"));
    }

    let metadata_path = metadata_path.canonicalize()?;

    which("tint")?;
    which("naga")?;

    let script_dir = PathBuf::from(env::var("SCRIPT_DIR")?);
    let bin_dir = script_dir.parent().unwrap().join("target/release");

    let status = Command::new("creduce")
        .env("WGSLREDUCE_SHADER_NAME", shader_path.file_name().unwrap())
        .env("WGSLREDUCE_METADATA_PATH", metadata_path)
        .env("WGSLREDUCE_SERVER", options.server)
        .env("WGSLREDUCE_BIN_PATH", bin_dir)
        .arg(script_dir.join("reduce-miscompilation.sh"))
        .arg(shader_path)
        .arg("--not-c")
        .status()?;

    if !status.success() {
        return Err(anyhow!("creduce did not complete successfully"));
    }

    Ok(())
}

fn which(bin: &str) -> anyhow::Result<PathBuf> {
    match which::which(bin) {
        Ok(path) => Ok(path),
        Err(e) => {
            if let Error::CannotFindBinaryPath = e {
                Err(anyhow!("cannot find executable path: {bin}"))
            } else {
                Err(e.into())
            }
        }
    }
}
