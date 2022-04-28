use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::rc::Rc;

use ast::{GlobalVarAttr, StorageClass};
use clap::Parser;
use common::{DataTypeExt, Resource, ResourceKind, ShaderMetadata};
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use wgslsmith::generator::Generator;
use wgslsmith::{Options, Preset};

fn main() -> io::Result<()> {
    let mut options = Options::parse();

    if let Some(preset) = &options.preset {
        match preset {
            Preset::Tint => {
                for name in ["countLeadingZeros", "countTrailingZeros"] {
                    if !options.enabled_fns.iter().any(|it| it == name) {
                        options.enabled_fns.push(name.to_owned());
                    }
                }
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
            let size = var.data_type.size();
            let (kind, init) = match qualifier.storage_class {
                StorageClass::Function => todo!(),
                StorageClass::Private => todo!(),
                StorageClass::WorkGroup => todo!(),
                StorageClass::Uniform => {
                    let init = (0..size).map(|_| rng.gen()).collect();
                    (ResourceKind::UniformBuffer, Some(init))
                }
                StorageClass::Storage => (ResourceKind::StorageBuffer, None),
            };

            let group = var.attrs.0.iter().find_map(|it| {
                if let GlobalVarAttr::Group(v) = it.attr {
                    Some(v as u32)
                } else {
                    None
                }
            });

            let binding = var.attrs.0.iter().find_map(|it| {
                if let GlobalVarAttr::Binding(v) = it.attr {
                    Some(v as u32)
                } else {
                    None
                }
            });

            resources.push(Resource {
                kind,
                group: group.expect("module variable must have group attribute"),
                binding: binding.expect("module variable must have binding attribute"),
                size,
                init,
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
