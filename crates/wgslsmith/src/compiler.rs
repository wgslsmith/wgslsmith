use std::fmt::Display;

use clap::ValueEnum;
use eyre::{eyre, Context};

#[derive(ValueEnum, Clone)]
pub enum Compiler {
    Tint,
    Naga,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Compiler::Tint => "tint",
            Compiler::Naga => "naga",
        };

        write!(f, "{val}")
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum Backend {
    Hlsl,
    Msl,
    Spirv,
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Backend::Hlsl => "hlsl",
            Backend::Msl => "msl",
            Backend::Spirv => "spirv",
        };

        write!(f, "{val}")
    }
}

impl Compiler {
    pub fn validate(&self, source: &str) -> eyre::Result<()> {
        match self {
            Compiler::Tint => validate_tint(source).wrap_err("tint validation failed"),
            Compiler::Naga => validate_naga(source).wrap_err("naga validation failed"),
        }
    }

    pub fn compile(&self, source: &str, backend: Backend) -> eyre::Result<String> {
        match self {
            Compiler::Tint => compile_tint(source, backend),
            Compiler::Naga => compile_naga(source, backend),
        }
    }
}

fn validate_naga(source: &str) -> eyre::Result<()> {
    use naga::front::wgsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let module = wgsl::parse_str(&source.replace("@stage(compute)", "@compute"))?;
    Validator::new(ValidationFlags::default(), Capabilities::all()).validate(&module)?;
    Ok(())
}

fn validate_tint(source: &str) -> eyre::Result<()> {
    tint::validate_shader(source)
        .then(|| ())
        .ok_or_else(|| eyre!("invalid wgsl"))
}

fn compile_naga(source: &str, backend: Backend) -> eyre::Result<String> {
    use naga::back::{hlsl, msl};
    use naga::front::wgsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    let module = wgsl::parse_str(&source.replace("@stage(compute)", "@compute"))?;
    let validation =
        Validator::new(ValidationFlags::default(), Capabilities::all()).validate(&module)?;

    let mut out = String::new();

    match backend {
        Backend::Hlsl => {
            hlsl::Writer::new(&mut out, &hlsl::Options::default()).write(&module, &validation)?;
        }
        Backend::Msl => {
            msl::Writer::new(&mut out).write(
                &module,
                &validation,
                &msl::Options::default(),
                &msl::PipelineOptions::default(),
            )?;
        }
        Backend::Spirv => todo!(),
    }

    Ok(out)
}

fn compile_tint(source: &str, backend: Backend) -> eyre::Result<String> {
    let out = match backend {
        Backend::Hlsl => tint::compile_shader_to_hlsl(source),
        Backend::Msl => tint::compile_shader_to_msl(source),
        Backend::Spirv => todo!(),
    };
    Ok(out)
}
