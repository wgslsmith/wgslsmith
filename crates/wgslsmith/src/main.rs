use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::rc::Rc;

use ast::{BuiltinFn, StorageClass, VarQualifier};
use clap::Parser;
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
                for builtin in [BuiltinFn::CountLeadingZeros, BuiltinFn::CountTrailingZeros] {
                    if !options.enabled_fns.iter().any(|it| *it == builtin) {
                        options.enabled_fns.push(builtin.to_owned());
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

    if !options.debug {
        let mut init_data = HashMap::new();

        for var in &shader.vars {
            if let Some(VarQualifier { storage_class, .. }) = &var.qualifier {
                if *storage_class != StorageClass::Uniform {
                    continue;
                }

                let type_desc = common::Type::try_from(&var.data_type).unwrap();

                let group = var.group_index().unwrap();
                let binding = var.binding_index().unwrap();

                let size = type_desc.buffer_size();
                let data: Vec<u8> = (0..size).map(|_| rng.gen()).collect();

                init_data.insert(format!("{group}:{binding}"), data);
            }
        }

        let init_data = serde_json::to_string(&init_data).unwrap();

        writeln!(output, "// {init_data}")?;
        writeln!(output, "// Seed: {seed}")?;
        writeln!(output)?;
    }

    if options.recondition {
        shader = reconditioner::recondition(shader);
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

        ast::writer::Writer::default()
            .write_module(&mut Output(&mut output), &shader)
            .unwrap();
    }

    Ok(())
}
