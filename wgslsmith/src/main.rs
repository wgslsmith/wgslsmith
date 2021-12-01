use clap::Parser;
use rand::prelude::StdRng;
use rand::rngs::OsRng;
use rand::SeedableRng;
use wgslsmith::generator::Generator;

#[derive(Parser)]
struct Options {
    #[clap(about = "Optional u64 to seed the random generator")]
    seed: Option<u64>,
    #[clap(short, long, about = "Print ast instead of WGSL code")]
    debug: bool,
}

fn main() {
    let options = Options::parse();
    let rng = match options.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_rng(OsRng::default())
            .expect("failed to seed random number generator from OsRng"),
    };

    let shader = Generator::new(rng).gen_module();

    if options.debug {
        println!("{:#?}", shader);
    } else {
        println!("{}", shader);
    }
}
