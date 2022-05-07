use std::env;
use std::process::{Command, Stdio};

use ast::Module;
use naga::valid::{Capabilities, ValidationFlags};

fn main() -> anyhow::Result<()> {
    let shader_name = env::var("WGSLREDUCE_SHADER_NAME")?;

    let source = std::fs::read_to_string(shader_name)?;
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

    if !exec_shader(&reconditioned)? {
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

fn exec_shader(source: &str) -> anyhow::Result<bool> {
    let server = env::var("WGSLREDUCE_SERVER")?;
    let metadata = std::fs::read_to_string(env::var("WGSLREDUCE_METADATA_PATH")?)?;
    let result = executor::exec_shader(&server, source, &metadata);
    Ok(result == 1)
}
