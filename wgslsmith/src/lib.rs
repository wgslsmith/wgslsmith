use std::hash::BuildHasher;

use clap::Parser;
use hashers::fx_hash::FxHasher;

pub mod generator;

#[derive(Parser)]
pub struct Options {
    #[clap(about = "Optional u64 to seed the random generator")]
    pub seed: Option<u64>,
    #[clap(short, long, about = "Print ast instead of WGSL code")]
    pub debug: bool,
    #[clap(
        long = "enable-fn",
        about = "Enable built-in functions that are disabled by default"
    )]
    pub enabled_fns: Vec<String>,
}

#[derive(Clone, Debug)]
struct BuildFxHasher;

impl BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::default()
    }
}
