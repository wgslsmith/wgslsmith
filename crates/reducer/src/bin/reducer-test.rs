use std::env;
use std::process::{Command, Stdio};

use anyhow::anyhow;
use ast::Module;
use naga::valid::{Capabilities, ValidationFlags};
use regex::Regex;

fn main() -> anyhow::Result<()> {
    let reduction_kind = env::var("WGSLREDUCE_KIND")?;
    let server = env::var("WGSLREDUCE_SERVER")?;

    let source = std::fs::read_to_string(env::var("WGSLREDUCE_SHADER_NAME")?)?;
    let metadata = std::fs::read_to_string(env::var("WGSLREDUCE_METADATA_PATH")?)?;

    match reduction_kind.as_str() {
        "crash" => reduce_crash(source, metadata, &server),
        "mismatch" => reduce_mismatch(source, metadata, &server),
        kind => Err(anyhow!("unknown reduction kind: {kind}")),
    }
}

fn reduce_crash(source: String, metadata: String, server: &str) -> anyhow::Result<()> {
    let configs = env::var("WGSLREDUCE_CONFIGURATIONS")?;
    let configs = configs.split(',').collect::<Vec<_>>();

    let regex = Regex::new(&env::var("WGSLREDUCE_REGEX")?)?;
    let should_recondition = env::var("WGSLREDUCE_RECONDITION").is_ok();

    let source = if should_recondition {
        recondition(parser::parse(&source))
    } else {
        source
    };

    let res = executor::exec_shader_with(server, &source, &metadata, configs);

    if res.exit_code != 101 || !regex.is_match(&res.output) {
        return Err(anyhow!("shader is not interesting"));
    }

    Ok(())
}

fn reduce_mismatch(source: String, metadata: String, server: &str) -> anyhow::Result<()> {
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

    if !exec_shader(&reconditioned, &metadata, server)? {
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
    if std::fs::write("reconditioned.wgsl", source).is_err() {
        return false;
    }

    let status = Command::new("tint")
        .arg("--validate")
        .arg("reconditioned.wgsl")
        .stdout(Stdio::null())
        .status();

    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn exec_shader(source: &str, metadata: &str, server: &str) -> anyhow::Result<bool> {
    Ok(executor::exec_shader(server, source, metadata).exit_code == 1)
}
