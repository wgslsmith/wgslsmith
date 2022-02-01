use std::hash::BuildHasher;

use clap::Parser;
use hashers::fx_hash::FxHasher;

pub mod generator;

#[derive(Parser)]
pub struct Options {
    /// Optional u64 to seed the random generator
    pub seed: Option<u64>,

    /// Print ast instead of WGSL code
    #[clap(short, long)]
    pub debug: bool,

    /// Enable built-in functions that are disabled by default
    #[clap(long = "enable-fn")]
    pub enabled_fns: Vec<String>,

    /// Logging configuration string (see https://docs.rs/tracing-subscriber/0.3.7/tracing_subscriber/struct.EnvFilter.html#directives)
    #[clap(long)]
    pub log: Option<String>,
}

#[derive(Clone, Debug)]
struct BuildFxHasher;

impl BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::default()
    }
}
