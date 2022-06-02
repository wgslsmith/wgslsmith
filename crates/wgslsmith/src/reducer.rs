use std::env;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use clap::{ArgEnum, Parser};
use eyre::eyre;
use regex::Regex;
use tap::Tap;
use which::Error;

#[derive(ArgEnum, Clone)]
enum Kind {
    Crash,
    Mismatch,
}

#[derive(Parser)]
pub struct Options {
    /// Type of bug that is being reduced.
    #[clap(arg_enum)]
    kind: Kind,

    /// Path to the WGSL shader file to reduce.
    shader: PathBuf,

    /// Path to the input data file.
    ///
    /// If not set, the program will look for a JSON file with the same name as the shader.
    input_data: Option<PathBuf>,

    /// Address of harness server.
    #[clap(short, long)]
    server: Option<String>,

    /// Config to use for reducing a crash.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(short, long)]
    config: Option<String>,

    /// Regex to match crash output against.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(long, default_value = "")]
    regex: Regex,

    /// Don't recondition shader before executing.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(long)]
    no_recondition: bool,

    /// Skip spawning harness server. If this is used then a server address must also be provided.
    #[clap(long, requires = "server")]
    no_spawn_server: bool,

    /// Disable logging from harness.
    #[clap(short, long)]
    quiet: bool,

    /// Enable debug mode for creduce.
    #[clap(long)]
    debug: bool,
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        if let Err(e) = self.0.kill() {
            eprintln!("failed to kill server: {e}");
        }
    }
}

pub fn run(options: Options) -> eyre::Result<()> {
    let shader_path = Path::new(&options.shader);
    if !shader_path.exists() {
        return Err(eyre!("shader at {shader_path:?} does not exist"));
    }

    let shader_path = shader_path.canonicalize()?;

    let input_path = if let Some(input_path) = options.input_data {
        input_path
    } else {
        shader_path
            .parent()
            .unwrap()
            .join(shader_path.file_stem().unwrap())
            .with_extension("json")
    };

    if !input_path.exists() {
        return Err(eyre!("file at {input_path:?} does not exist"));
    }

    let metadata_path = input_path.canonicalize()?;

    which("tint")?;
    which("naga")?;

    let (handle, address) = if options.no_spawn_server {
        (None, options.server.unwrap())
    } else {
        println!("> spawning harness server");

        let mut harness = Command::new(env::current_exe().unwrap())
            .args(["harness", "serve"])
            .tap_mut(|cmd| {
                if let Some(address) = options.server.as_deref() {
                    cmd.args(["-a", address]);
                }
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = BufReader::new(harness.stdout.take().unwrap()).lines();
        let stderr = BufReader::new(harness.stderr.take().unwrap()).lines();

        println!("> waiting for server to start listening");
        let mut address = None;
        for line in &mut stdout {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    eprintln!("failed to read from harness server stdout: {e}");
                    break;
                }
            };

            if !options.quiet {
                println!("{line}");
            }

            if let Some(value) = line.strip_prefix("Server listening at ") {
                address = Some(value.trim().to_owned());
                break;
            }
        }

        let thread = std::thread::spawn(move || {
            let stdout_thread = std::thread::spawn(move || {
                for line in stdout.flatten() {
                    if !options.quiet {
                        println!("{line}");
                    }
                }
            });

            let stderr_thread = std::thread::spawn(move || {
                for line in stderr.flatten() {
                    if !options.quiet {
                        println!("{line}");
                    }
                }
            });

            stdout_thread.join().unwrap();
            stderr_thread.join().unwrap();
        });

        let address = address.ok_or_else(|| eyre!("failed to read harness server address"))?;

        println!("> detected harness server listening at {address}");

        (Some((Server(harness), thread)), address)
    };

    let interestingness_test =
        PathBuf::from(env::var("WGSLSMITH_ROOT").unwrap()).join("scripts/reducer-test.sh");

    let mut cmd = Command::new("creduce");

    match options.kind {
        Kind::Crash => {
            let config = match options.config {
                Some(config) => config,
                None => return Err(eyre!("a configuration is required when reducing a crash")),
            };

            cmd.env("WGSLREDUCE_KIND", "crash")
                .env("WGSLREDUCE_CONFIGURATIONS", config)
                .env("WGSLREDUCE_REGEX", options.regex.as_str());

            if !options.no_recondition {
                cmd.env("WGSLREDUCE_RECONDITION", "1");
            }
        }
        Kind::Mismatch => {
            cmd.env("WGSLREDUCE_KIND", "mismatch");
        }
    }

    let status = cmd
        .env("WGSLREDUCE_SHADER_NAME", shader_path.file_name().unwrap())
        .env("WGSLREDUCE_METADATA_PATH", metadata_path)
        .env("WGSLREDUCE_SERVER", address)
        .arg(interestingness_test)
        .arg(shader_path)
        .arg("--not-c")
        .tap_mut(|cmd| {
            if options.debug {
                cmd.arg("--debug");
            }
        })
        .status()?;

    if let Some((handle, thread)) = handle {
        drop(handle);
        thread.join().unwrap();
    }

    if !status.success() {
        return Err(eyre!("creduce did not complete successfully"));
    }

    Ok(())
}

fn which(bin: &str) -> eyre::Result<PathBuf> {
    match which::which(bin) {
        Ok(path) => Ok(path),
        Err(e) => {
            if let Error::CannotFindBinaryPath = e {
                Err(eyre!("cannot find executable path: {bin}"))
            } else {
                Err(e.into())
            }
        }
    }
}
