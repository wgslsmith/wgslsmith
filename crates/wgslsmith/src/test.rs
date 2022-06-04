use std::env;
use std::ffi::CString;
use std::io::Write;
use std::process::{Command, Stdio};

use ast::Module;
use eyre::eyre;
use naga::valid::{Capabilities, ValidationFlags};
use regex::Regex;

use crate::executor;

enum Harness {
    Local,
    Remote(String),
}

pub fn run() -> eyre::Result<()> {
    let reduction_kind = env::var("WGSLREDUCE_KIND")?;

    let harness = if let Ok(server) = env::var("WGSLREDUCE_SERVER") {
        Harness::Remote(server)
    } else {
        Harness::Local
    };

    let source = std::fs::read_to_string(env::var("WGSLREDUCE_SHADER_NAME")?)?;
    let metadata = std::fs::read_to_string(env::var("WGSLREDUCE_METADATA_PATH")?)?;

    match reduction_kind.as_str() {
        "crash" => reduce_crash(source, metadata, &harness),
        "mismatch" => reduce_mismatch(source, metadata, &harness),
        kind => Err(eyre!("unknown reduction kind: {kind}")),
    }
}

fn reduce_crash(source: String, metadata: String, harness: &Harness) -> eyre::Result<()> {
    let configs = env::var("WGSLREDUCE_CONFIGURATIONS")?;
    let configs = configs.split(',').collect::<Vec<_>>();

    let regex = Regex::new(&env::var("WGSLREDUCE_REGEX")?)?;
    let should_recondition = env::var("WGSLREDUCE_RECONDITION").is_ok();

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
                .spawn()?;
            write!(child.stdin.take().unwrap(), "{source}")?;
            let output = child.wait_with_output()?;
            Ok(output.status.code().unwrap() != 101
                || !regex.is_match(&String::from_utf8_lossy(&output.stdout)))
        }
        Harness::Remote(server) => {
            let res = executor::exec_shader_with(server, source, metadata, configs)?;
            Ok(res.exit_code != 101 || !regex.is_match(&res.output))
        }
    }
}
