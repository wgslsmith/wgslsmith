mod gen;

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use ast::{BuiltinFn, StorageClass, VarQualifier};
use clap::Parser;
use eyre::{bail, eyre};
use hashers::fx_hash::FxHasher;
use reconditioner::evaluator;

pub use gen::{builtins, Generator};
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Preset for crash-testing Tint.
    Tint,
}

impl FromStr for Preset {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tint" => Ok(Preset::Tint),
            _ => Err("invalid preset - must be one of {tint}"),
        }
    }
}

#[derive(Parser)]
pub struct Options {
    /// Optional u64 to seed the random generator
    #[clap(action)]
    pub seed: Option<u64>,

    /// Print ast instead of WGSL code
    #[clap(short, long, action)]
    pub debug: bool,

    /// Enable built-in functions that are disabled by default
    #[clap(long = "enable-fn", action)]
    pub enabled_fns: Vec<BuiltinFn>,

    /// Whether to enable generating pointers.
    #[clap(long, action)]
    pub enable_pointers: bool,

    /// Skips the static pointer aliasing checks.
    ///
    /// This is only useful if reconditioning and pointer support is enabled.
    #[clap(long, action)]
    pub skip_pointer_checks: bool,

    /// Logging configuration string (see https://docs.rs/tracing-subscriber/0.3.7/tracing_subscriber/struct.EnvFilter.html#directives)
    #[clap(long, action)]
    pub log: Option<String>,

    /// Minimum number of statements to generate in function bodies
    #[clap(long, action, default_value = "5")]
    pub fn_min_stmts: u32,

    /// Maximum number of statements to generate in function bodies
    #[clap(long, action, default_value = "5")]
    pub fn_max_stmts: u32,

    /// Minimum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, action, default_value = "0")]
    pub block_min_stmts: u32,

    /// Maximum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, action, default_value = "5")]
    pub block_max_stmts: u32,

    /// Maximum nested block depth
    #[clap(long, action, default_value = "3")]
    pub max_block_depth: u32,

    /// Maximum number of function to generate
    #[clap(long, action, default_value = "5")]
    pub max_fns: u32,

    /// Minimum number of structs to generate (excluding input and output)
    #[clap(long, action, default_value = "1")]
    pub min_structs: u32,

    /// Maximum number of structs to generate (excluding input and output)
    #[clap(long, action, default_value = "5")]
    pub max_structs: u32,

    /// Minimum number of members allowed in a struct
    #[clap(long, action, default_value = "1")]
    pub min_struct_members: u32,

    /// Maximum number of members allowed in a struct
    #[clap(long, action, default_value = "5")]
    pub max_struct_members: u32,

    /// Preset options configuration. Individual options may still be overridden.
    #[clap(long, action)]
    pub preset: Option<Preset>,

    /// Recondition the resulting program to remove UB
    #[clap(long, action)]
    pub recondition: bool,

    /// Path to output file (use `-` for stdout)
    #[clap(short, long, action, default_value = "-")]
    pub output: String,
}

pub fn run(mut options: Options) -> eyre::Result<()> {
    if let Some(preset) = &options.preset {
        match preset {
            Preset::Tint => {
                for builtin in builtins::TINT_EXTRAS {
                    if !options.enabled_fns.iter().any(|it| it == builtin) {
                        options.enabled_fns.push(builtin.to_owned());
                    }
                }

                options.enable_pointers = true;
                options.skip_pointer_checks = true;
                options.recondition = true;
            }
        }
    }

    let options = Rc::new(options);

    tracing_subscriber::fmt()
        .compact()
        .with_span_events(FmtSpan::ACTIVE)
        .with_target(true)
        .with_writer(io::stderr)
        .with_ansi(false)
        .with_env_filter(if let Some(log) = &options.log {
            EnvFilter::from(log)
        } else {
            EnvFilter::from_default_env()
        })
        .init();

    let seed = match options.seed {
        Some(seed) => seed,
        None => OsRng.gen(),
    };

    tracing::info!("generating shader from seed: {}", seed);

    let mut rng = StdRng::seed_from_u64(seed);
    let mut shader = Generator::new(&mut rng, options.clone()).gen_module();

    if options.recondition {
        if options.enable_pointers
            && !options.skip_pointer_checks
            && !reconditioner::analysis::analyse(&shader)
        {
            bail!("rejected shader due to possible invalid aliasing");
        }

        let concrete_shader = evaluator::concretize(shader);

        shader = reconditioner::recondition_with(
            concrete_shader,
            reconditioner::Options {
                only_loops: options.preset == Some(Preset::Tint),
            },
        );
    }

    let mut output: Box<dyn io::Write> = if options.output == "-" {
        Box::new(io::stdout())
    } else {
        if let Some(dir) = Path::new(&options.output).parent() {
            std::fs::create_dir_all(dir)?;
        }
        Box::new(BufWriter::new(File::create(&options.output)?))
    };

    if !options.debug {
        let mut init_data = HashMap::new();

        for var in &shader.vars {
            if let Some(VarQualifier { storage_class, .. }) = &var.qualifier {
                if *storage_class != StorageClass::Uniform {
                    continue;
                }

                let type_desc = common::Type::try_from(&var.data_type).map_err(|e| eyre!(e))?;

                let group = var.group_index().unwrap();
                let binding = var.binding_index().unwrap();

                let size = type_desc.buffer_size();
                let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

                init_data.insert(format!("{group}:{binding}"), data);
            }
        }

        let init_data = serde_json::to_string(&init_data)?;

        writeln!(output, "// {init_data}")?;
        writeln!(output, "// Seed: {seed}")?;
        writeln!(output)?;
    }

    if options.debug {
        writeln!(output, "{shader:#?}")?;
    } else {
        struct Output<'a>(&'a mut dyn std::io::Write);

        impl std::fmt::Write for Output<'_> {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                self.0.write_all(s.as_bytes()).unwrap();
                Ok(())
            }
        }

        ast::writer::Writer::default().write_module(&mut Output(&mut output), &shader)?;
    }

    Ok(())
}
