use clap::Parser;
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::{Rng, SeedableRng};
use wgslsmith::generator::Generator;
use wgslsmith::Options;

fn main() {
    env_logger::init();

    let options = Options::parse();
    let seed = match options.seed {
        Some(seed) => seed,
        None => OsRng::default().gen(),
    };

    log::info!("generating shader from seed: {}", seed);

    let rng = StdRng::seed_from_u64(seed);
    let shader = Generator::new(rng).gen_module(&options);

    if options.debug {
        println!("{:#?}", shader);
    } else {
        println!("// Seed: {}\n", seed);
        println!("{}", shader);
    }
}
