use std::env;
use std::ffi::CString;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ast::Module;
use clap::Parser;
use eyre::eyre;
use naga::valid::{Capabilities, ValidationFlags};
use regex::Regex;

use crate::executor;
use crate::reducer::Kind;

enum Harness {
    Local,
    Remote(String),
}

#[derive(Parser)]
pub struct Options {
    #[clap(arg_enum)]
    kind: Kind,

    shader: PathBuf,

    input_data: PathBuf,

    #[clap(long)]
    server: Option<String>,

    #[clap(flatten)]
    crash_opts: CrashOptions,
}

#[derive(Parser)]
pub struct CrashOptions {
    #[clap(long)]
    config: Option<String>,

    #[clap(long)]
    regex: Option<Regex>,

    #[clap(long)]
    no_recondition: bool,
}

pub fn run(options: Options) -> eyre::Result<()> {
    let source = std::fs::read_to_string(&options.shader)?;
    let metadata = std::fs::read_to_string(&options.input_data)?;

    let harness = if let Some(server) = options.server {
        Harness::Remote(server)
    } else {
        Harness::Local
    };

    match options.kind {
        Kind::Crash => reduce_crash(options.crash_opts, source, metadata, &harness),
        Kind::Mismatch => reduce_mismatch(source, metadata, &harness),
    }
}

fn reduce_crash(
    options: CrashOptions,
    source: String,
    metadata: String,
    harness: &Harness,
) -> eyre::Result<()> {
    let config = options.config.unwrap();
    let configs = vec![config.as_str()];

    let regex = options.regex.unwrap();
    let should_recondition = !options.no_recondition;

    let source = if should_recondition {
        recondition(parser::parse(&source))
    } else {
        source
    };

    if !exec_for_crash(&source, &metadata, &regex, harness, configs)? {
        return Err(eyre!("shader is not interesting"));
    }

    Ok(())
}

fn reduce_mismatch(source: String, metadata: String, server: &Harness) -> eyre::Result<()> {
    let module = parser::parse(&source);
    let reconditioned = recondition(module);

    if !validate_naga(&reconditioned) {
        eprintln!("naga validation failed");
        std::process::exit(1);
    }

    if !validate_tint(&reconditioned) {
        eprintln!("tint validation failed");
        std::process::exit(1);
    }

    if !exec_for_mismatch(&reconditioned, &metadata, server)? {
        eprintln!("shader is not interesting");
        std::process::exit(1);
    }

    Ok(())
}

fn recondition(module: Module) -> String {
    let reconditioned = reconditioner::recondition(module);
    let mut formatted = String::new();

    ast::writer::Writer::default()
        .write_module(&mut formatted, &reconditioned)
        .unwrap();

    formatted
}

fn validate_naga(source: &str) -> bool {
    let module = match naga::front::wgsl::parse_str(&source.replace("@stage(compute)", "@compute"))
    {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            return false;
        }
    };

    let validation = naga::valid::Validator::new(ValidationFlags::default(), Capabilities::all())
        .validate(&module);

    if let Err(e) = validation {
        eprintln!("{e:?}");
        return false;
    }

    true
}

fn validate_tint(source: &str) -> bool {
    let source = CString::new(source).unwrap();
    unsafe { tint::validate_shader(source.as_ptr()) }
}

fn exec_for_mismatch(source: &str, metadata: &str, harness: &Harness) -> eyre::Result<bool> {
    match harness {
        Harness::Local => {
            let mut child = Command::new(env::current_exe().unwrap())
                .args(["harness", "run", "-", metadata])
                .stdin(Stdio::piped())
                .spawn()?;
            write!(child.stdin.take().unwrap(), "{source}")?;
            Ok(child.wait()?.code().unwrap() == 1)
        }
        Harness::Remote(server) => {
            Ok(executor::exec_shader(server, source, metadata)?.exit_code == 1)
        }
    }
}

fn exec_for_crash(
    source: &str,
    metadata: &str,
    regex: &Regex,
    harness: &Harness,
    configs: Vec<&str>,
) -> eyre::Result<bool> {
    match harness {
        Harness::Local => {
            let mut child = Command::new(env::current_exe().unwrap())
                .args(["harness", "run", "-", metadata])
                .args(configs.into_iter().flat_map(|c| ["-c", c]))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            write!(child.stdin.take().unwrap(), "{source}")?;
            let output = child.wait_with_output()?;
            Ok(output.status.code().unwrap() == 101
                && regex.is_match(&String::from_utf8_lossy(&output.stderr)))
        }
        Harness::Remote(server) => {
            let res = executor::exec_shader_with(server, source, metadata, configs)?;
            Ok(res.exit_code == 101 && regex.is_match(&res.output))
        }
    }
}
