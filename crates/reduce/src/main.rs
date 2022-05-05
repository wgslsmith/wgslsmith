use std::env;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    #[clap(short, long)]
    server: Option<String>,
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
    let harness_bin_dir = script_dir.parent().unwrap().join("harness/target/release");

    let (handle, address) = if let Some(address) = options.server {
        (None, address)
    } else {
        let mut harness = Command::new(harness_bin_dir.join("harness-server"))
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = harness.stdout.take().unwrap();
        let mut stdout = BufReader::new(stdout).lines();

        let mut address = None;
        for line in &mut stdout {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    eprintln!("failed to read from harness server stdout: {e}");
                    break;
                }
            };

            if let Some(value) = line.strip_prefix("Server listening at ") {
                address = Some(value.trim().to_owned());
                break;
            }
        }

        let thread = std::thread::spawn(move || {
            for line in stdout.flatten() {
                println!("[HARNESS] {line}");
            }
        });

        let address = address.ok_or_else(|| anyhow!("failed to read harness server address"))?;

        println!("> detected harness server running at {address}");

        (Some((harness, thread)), address)
    };

    let status = Command::new("creduce")
        .env("WGSLREDUCE_SHADER_NAME", shader_path.file_name().unwrap())
        .env("WGSLREDUCE_METADATA_PATH", metadata_path)
        .env("WGSLREDUCE_SERVER", address)
        .env("WGSLREDUCE_BIN_PATH", bin_dir)
        .arg(script_dir.join("reduce-miscompilation.sh"))
        .arg(shader_path)
        .arg("--not-c")
        .arg("--debug")
        .status()?;

    if !status.success() {
        return Err(anyhow!("creduce did not complete successfully"));
    }

    if let Some((mut handle, thread)) = handle {
        handle.kill()?;
        thread.join().unwrap();
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
