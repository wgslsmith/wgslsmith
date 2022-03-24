use std::hash::BuildHasher;

use ast::AttrStyle;
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

    /// Minimum number of statements to generate in function bodies
    #[clap(long, default_value = "5")]
    pub fn_min_stmts: u32,

    /// Maximum number of statements to generate in function bodies
    #[clap(long, default_value = "10")]
    pub fn_max_stmts: u32,

    /// Minimum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, default_value = "0")]
    pub block_min_stmts: u32,

    /// Maximum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, default_value = "10")]
    pub block_max_stmts: u32,

    /// Maximum nested block depth
    #[clap(long, default_value = "5")]
    pub max_block_depth: u32,

    /// Maximum number of function to generate
    #[clap(long, default_value = "10")]
    pub max_fns: u32,

    /// Minimum number of structs to generate (excluding input and output)
    #[clap(long, default_value = "1")]
    pub min_structs: u32,

    /// Maximum number of structs to generate (excluding input and output)
    #[clap(long, default_value = "10")]
    pub max_structs: u32,

    /// Minimum number of members allowed in a struct
    #[clap(long, default_value = "1")]
    pub min_struct_members: u32,

    /// Maximum number of members allowed in a struct
    #[clap(long, default_value = "10")]
    pub max_struct_members: u32,

    /// Enabled attribute styles {java, cpp} (if multiple styles are enabled, they will be selected from randomly)
    #[clap(long, default_value = "java")]
    pub attribute_style: Vec<AttrStyle>,

    /// Recondition the resulting program to remove UB
    #[clap(long)]
    pub recondition: bool,

    /// Path to output file (use `-` for stdout)
    #[clap(short, long, default_value = "-")]
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
