use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ast::Module;
use clap::Parser;
use eyre::eyre;
use naga::valid::{Capabilities, ValidationFlags};
use regex::Regex;
use tempfile::NamedTempFile;

use crate::config::Config;
use crate::executor;
use crate::reducer::{Backend, Compiler, Kind};

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
    #[clap(long, conflicts_with("compiler"))]
    config: Option<String>,

    #[clap(long, arg_enum, requires("backend"))]
    compiler: Option<Compiler>,

    #[clap(long, arg_enum)]
    backend: Option<Backend>,

    #[clap(long, required_if_eq("kind", "crash"))]
    regex: Option<Regex>,

    #[clap(long)]
    no_recondition: bool,
}

pub fn run(config: &Config, options: Options) -> eyre::Result<()> {
    let source = std::fs::read_to_string(&options.shader)?;
    let metadata = std::fs::read_to_string(&options.input_data)?;

    let harness = if let Some(server) = options.server {
        Harness::Remote(server)
    } else {
        Harness::Local
    };

    match options.kind {
        Kind::Crash => reduce_crash(config, options.crash_opts, source, metadata, &harness)?,
        Kind::Mismatch => reduce_mismatch(source, metadata, &harness)?,
    }

    println!("interesting :)");

    Ok(())
}

fn reduce_crash(
    config: &Config,
    options: CrashOptions,
    source: String,
    metadata: String,
    harness: &Harness,
) -> eyre::Result<()> {
    let regex = options.regex.unwrap();
    let should_recondition = !options.no_recondition;

    let source = if should_recondition {
        recondition(parser::parse(&source))
    } else {
        source
    };

    let interesting = if let Some(config) = options.config {
        let configs = vec![config.as_str()];
        exec_for_crash(&source, &metadata, &regex, harness, configs)?
    } else {
        let compiler = options.compiler.unwrap();
        let backend = options.backend.unwrap();
        let compiled = match compiler {
            Compiler::Naga => compile_naga(&source, backend)?,
            Compiler::Tint => compile_tint(&source, backend)?,
        };

        match backend {
            Backend::Hlsl => validate_hlsl(config, &compiled, &regex)?,
            Backend::Msl => validate_metal(config, &compiled, &regex)?,
            Backend::Spirv => todo!(),
        }
    };

    if !interesting {
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
    tint::validate_shader(source)
}

fn compile_naga(source: &str, backend: Backend) -> eyre::Result<String> {
    let module = naga::front::wgsl::parse_str(&source.replace("@stage(compute)", "@compute"))?;
    let validation = naga::valid::Validator::new(ValidationFlags::default(), Capabilities::all())
        .validate(&module)?;

    let mut out = String::new();

    match backend {
        Backend::Hlsl => {
            naga::back::hlsl::Writer::new(&mut out, &naga::back::hlsl::Options::default())
                .write(&module, &validation)?;
        }
        Backend::Msl => {
            naga::back::msl::Writer::new(&mut out).write(
                &module,
                &validation,
                &naga::back::msl::Options::default(),
                &naga::back::msl::PipelineOptions::default(),
            )?;
        }
        Backend::Spirv => todo!(),
    }

    Ok(out)
}

fn compile_tint(source: &str, backend: Backend) -> eyre::Result<String> {
    let out = match backend {
        Backend::Hlsl => tint::compile_shader_to_hlsl(source),
        Backend::Msl => todo!(),
        Backend::Spirv => todo!(),
    };
    Ok(out)
}

fn validate_hlsl(config: &Config, hlsl: &str, regex: &Regex) -> eyre::Result<bool> {
    let mut file = NamedTempFile::new_in(env::current_dir()?)?;
    write!(file, "{hlsl}")?;
    file.flush()?;

    let root = PathBuf::from(env::var("WGSLSMITH_ROOT")?);
    let fxc = root.join("tools/fxc.exe");
    let (mut cmd, path) = if config.validator.fxc.use_wine {
        println!("running fxc with wine");
        let mut cmd = Command::new("wine");
        cmd.arg(fxc);
        let path = PathBuf::from(format!("z:/{}", file.path().canonicalize()?.display()));
        (cmd, path)
    } else {
        let cmd = Command::new(fxc);
        let path = pathdiff::diff_paths(file.path(), env::current_dir()?).unwrap();
        (cmd, path)
    };

    let output = cmd
        .args(["/T", "cs_5_1", "/E", "main"])
        .arg(path)
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    Ok(output.status.code().unwrap() != 0 && regex.is_match(&stderr))
}

fn validate_metal(config: &Config, metal: &str, regex: &Regex) -> eyre::Result<bool> {
    let mut file = NamedTempFile::new_in(env::current_dir()?)?;
    write!(file, "{metal}")?;
    file.flush()?;

    let output = Command::new(config.validator.metal.path()?)
        .args(["-x", "metal"])
        .args(["-o", "NUL"])
        .arg("-std=osx-metal2.0")
        .arg("-c")
        .arg(file.path())
        .stderr(Stdio::piped())
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    Ok(output.status.code().unwrap() != 0 && regex.is_match(&stderr))
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
