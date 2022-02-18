use std::rc::Rc;

use clap::Parser;
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use wgslsmith::generator::Generator;
use wgslsmith::Options;

fn main() {
    let options = Rc::new(Options::parse());

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_target(true)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(if let Some(log) = &options.log {
            EnvFilter::from(log)
        } else {
            EnvFilter::from_default_env()
        })
        .init();

    let seed = match options.seed {
        Some(seed) => seed,
        None => OsRng::default().gen(),
    };

    tracing::info!("generating shader from seed: {}", seed);

    let rng = StdRng::seed_from_u64(seed);
    let mut shader = Generator::new(rng).gen_module(options.clone());

    if !options.debug {
        println!("// Seed: {}\n", seed);
    }

    if options.recondition {
        let result = reconditioner::recondition(shader);

        // If the program contains any loops, write the loop_counters array declaration
        if !options.debug && result.loop_count > 0 {
            println!(
                "var<private> LOOP_COUNTERS: array<u32, {}>;\n",
                result.loop_count
            );
        }

        shader = result.ast;
    }

    if options.debug {
        println!("{:#?}", shader);
    } else {
        println!("{}", shader);
    }
}
