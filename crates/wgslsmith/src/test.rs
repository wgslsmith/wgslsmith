use std::path::PathBuf;

use ast::Module;
use clap::Parser;
use eyre::eyre;
use harness_types::ConfigId;
use regex::Regex;

use crate::compiler::{Backend, Compiler};
use crate::config::Config;
use crate::harness_runner::{ExecutionResult, Harness};
use crate::reducer::ReductionKind;
use crate::{harness_runner, validator};

#[derive(Parser)]
pub struct Options {
    #[clap(action, action)]
    kind: ReductionKind,

    #[clap(action)]
    shader: PathBuf,

    #[clap(action)]
    input_data: Option<PathBuf>,

    #[clap(long, action)]
    server: Option<String>,

    #[clap(flatten)]
    crash_options: CrashOptions,

    #[clap(short, long, action)]
    quiet: bool,
}

#[derive(Parser)]
pub struct CrashOptions {
    #[clap(long, action, conflicts_with("compiler"))]
    config: Option<ConfigId>,

    #[clap(long, value_enum, action, requires("backend"))]
    compiler: Option<Compiler>,

    #[clap(long, value_enum, action)]
    backend: Option<Backend>,

    #[clap(long, action, required_if_eq("kind", "crash"))]
    regex: Option<Regex>,

    #[clap(long, action)]
    no_recondition: bool,
}

pub fn run(config: &Config, options: Options) -> eyre::Result<()> {
    let source = std::fs::read_to_string(&options.shader)?;

    let input_path = if let Some(input_path) = options.input_data {
        input_path
    } else {
        let mut try_path = options
            .shader
            .parent()
            .unwrap()
            .join(options.shader.file_stem().unwrap())
            .with_extension("json");

        if !try_path.exists() {
            try_path = options.shader.parent().unwrap().join("inputs.json");
        }

        if !try_path.exists() {
            try_path = options
                .shader
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("inputs.json");
        }

        if !try_path.exists() {
            return Err(eyre!(
                "couldn't determine path to inputs file, pass one explicitly"
            ));
        }

        try_path
    };

    let metadata = std::fs::read_to_string(&input_path)?;

    let harness = if let Some(server) = options.server {
        Harness::Remote(server)
    } else {
        Harness::Local(
            config
                .harness
                .path
                .clone()
                .map(Ok)
                .unwrap_or_else(std::env::current_exe)?,
        )
    };

    match options.kind {
        ReductionKind::Crash => reduce_crash(
            config,
            options.crash_options,
            source,
            metadata,
            &harness,
            options.quiet,
        )?,
        ReductionKind::Mismatch => reduce_mismatch(source, metadata, &harness, options.quiet)?,
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
    quiet: bool,
) -> eyre::Result<()> {
    let regex = options.regex.unwrap();
    let should_recondition = !options.no_recondition;

    let source = if should_recondition {
        recondition(parser::parse(&source))
    } else {
        source
    };

    let interesting = if let Some(config) = options.config {
        let result =
            harness_runner::exec_shader(harness, Some(config), &source, &metadata, |line| {
                if !quiet {
                    println!("{line}");
                }
            })?;

        eprintln!("{result:?}");

        matches!(result, ExecutionResult::Crash(output) if regex.is_match(&output))
    } else {
        let compiler = options.compiler.unwrap();
        let backend = options.backend.unwrap();
        let compiled = compiler.compile(&source, backend)?;

        match backend {
            Backend::Hlsl => {
                remote_validate(config, &compiled, validator::Backend::Hlsl, &regex, quiet)?
            }
            Backend::Msl => {
                remote_validate(config, &compiled, validator::Backend::Msl, &regex, quiet)?
            }
            Backend::Spirv => todo!(),
        }
    };

    if !interesting {
        return Err(eyre!("shader is not interesting"));
    }

    Ok(())
}

fn reduce_mismatch(
    source: String,
    metadata: String,
    harness: &Harness,
    quiet: bool,
) -> eyre::Result<()> {
    let module = parser::parse(&source);
    let reconditioned = recondition(module);

    Compiler::Naga.validate(&reconditioned)?;
    Compiler::Tint.validate(&reconditioned)?;

    let result = harness_runner::exec_shader(harness, None, &reconditioned, &metadata, |line| {
        if !quiet {
            println!("{line}");
        }
    })?;

    if result != ExecutionResult::Mismatch {
        return Err(eyre!("shader is not interesting"));
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

fn remote_validate(
    config: &Config,
    source: &str,
    backend: validator::Backend,
    regex: &Regex,
    quiet: bool,
) -> eyre::Result<bool> {
    if !quiet {
        println!("[SOURCE]");
        println!("{source}");
    }

    let server = config.validator.server()?;
    let result = validator::validate(server, backend, source.to_owned())?;

    let is_interesting = match result {
        validator::ValidateResponse::Success => false,
        validator::ValidateResponse::Failure(err) => {
            if !quiet {
                println!("-----");
                println!("{err}");
            }
            regex.is_match(&err)
        }
    };

    Ok(is_interesting)
}
