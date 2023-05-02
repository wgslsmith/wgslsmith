mod data_race_gen;

use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasher;
use std::io::{self, BufWriter};
use std::path::Path;
use std::rc::Rc;

use ast::{BuiltinFn, StorageClass, VarQualifier};
use clap::Parser;
use eyre::{bail, eyre};
use hashers::fx_hash::FxHasher;

pub use data_race_gen::{Generator};
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};

#[derive(Parser)]
pub struct Options {
    /// Optional u64 to seed the random generator
    #[clap(action)]
    pub seed: Option<u64>,

    /// Print ast instead of WGSL code
    #[clap(short, long, action)]
    pub debug: bool,

    /// Logging configuration string (see https://docs.rs/tracing-subscriber/0.3.7/tracing_subscriber/struct.EnvFilter.html#directives)
    #[clap(long, action)]
    pub log: Option<String>,

    /// Number of literals to generate
    #[clap(long, action, default_value = "5")]
    pub num_lits: u32,

    /// Minimum number of statements to generate
    #[clap(long, action, default_value = "5")]
    pub min_stmts: u32,

    /// Maximum number of statements to generate
    #[clap(long, action, default_value = "5")]
    pub max_stmts: u32,

    /// Minimum number of local variables to generate
    #[clap(long, action, default_value = "5")]
    pub min_vars: u32,

    /// Maximum number of local variables to generate
    #[clap(long, action, default_value = "5")]
    pub max_vars: u32,

    /// Number of memory locations associated with each thread 
    #[clap(long, action, default_value = "5")]
    pub locs_per_thread: u32,

    /// Path to output file (use `-` for stdout)
    #[clap(short, long, action, default_value = "-")]
    pub output: String,
}

#[derive(Clone, Debug)]
struct BuildFxHasher;

impl BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::default()
    }
}

pub fn run(mut options: Options) -> eyre::Result<()> {
    let options = Rc::new(options);

    let seed = match options.seed {
        Some(seed) => seed,
        None => OsRng.gen(),
    };

    tracing::info!("generating shader from seed: {}", seed);

    let mut rng = StdRng::seed_from_u64(seed);
    let mut shader = Generator::new(&mut rng, options.clone()).gen_module();

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

        impl<'a> std::fmt::Write for Output<'a> {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                self.0.write_all(s.as_bytes()).unwrap();
                Ok(())
            }
        }

        ast::writer::Writer::default().write_module(&mut Output(&mut output), &shader)?;
    }

    Ok(())
}
