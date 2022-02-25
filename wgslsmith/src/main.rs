use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::rc::Rc;

use ast::StorageClass;
use clap::Parser;
use common::{Resource, ResourceKind, ShaderMetadata};
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use wgslsmith::generator::Generator;
use wgslsmith::Options;

fn main() -> io::Result<()> {
    let options = Rc::new(Options::parse());

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

    let rng = StdRng::seed_from_u64(seed);
    let mut shader = Generator::new(rng, options.clone()).gen_module();
    let mut output: Box<dyn io::Write> = if options.output == "-" {
        Box::new(io::stdout())
    } else {
        if let Some(dir) = Path::new(&options.output).parent() {
            std::fs::create_dir_all(dir)?;
        }
        Box::new(BufWriter::new(File::create(&options.output)?))
    };

    let mut resources = vec![];

    for var in &shader.vars {
        if let Some(qualifier) = &var.qualifier {
            let kind = match qualifier.storage_class {
                StorageClass::Function => todo!(),
                StorageClass::Private => todo!(),
                StorageClass::WorkGroup => todo!(),
                StorageClass::Uniform => ResourceKind::UniformBuffer,
                StorageClass::Storage => ResourceKind::StorageBuffer,
            };

            resources.push(Resource {
                kind,
                binding: 0,
                group: 0,
                description: var.data_type.clone(),
            })
        }
    }

    let meta = ShaderMetadata { resources };

    if !options.debug {
        writeln!(output, "// {}", serde_json::to_string(&meta).unwrap())?;
        writeln!(output, "// Seed: {}\n", seed)?;
    }

    if options.recondition {
        let result = reconditioner::recondition(shader);

        // If the program contains any loops, write the loop_counters array declaration
        if !options.debug && result.loop_count > 0 {
            writeln!(
                output,
                "var<private> LOOP_COUNTERS: array<u32, {}>;\n",
                result.loop_count
            )?;
        }

        writeln!(
            output,
            "{}",
            include_str!("../../reconditioner/src/prelude.wgsl")
        )?;

        shader = result.ast;
    }

    if options.debug {
        writeln!(output, "{:#?}", shader)?;
    } else {
        writeln!(output, "{}", shader)?;
    }

    Ok(())
}
