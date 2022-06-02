mod gen;

use std::hash::BuildHasher;
use std::str::FromStr;

use ast::BuiltinFn;
use clap::Parser;
use hashers::fx_hash::FxHasher;

pub use gen::{builtins, Generator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Preset for crash-testing Tint.
    Tint,
}

impl FromStr for Preset {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tint" => Ok(Preset::Tint),
            _ => Err("invalid preset - must be one of {tint}"),
        }
    }
}

#[derive(Parser)]
pub struct Options {
    /// Optional u64 to seed the random generator
    pub seed: Option<u64>,

    /// Print ast instead of WGSL code
    #[clap(short, long)]
    pub debug: bool,

    /// Enable built-in functions that are disabled by default
    #[clap(long = "enable-fn")]
    pub enabled_fns: Vec<BuiltinFn>,

    /// Whether to enable generating pointers.
    #[clap(long)]
    pub enable_pointers: bool,

    /// Skips the static pointer aliasing checks.
    ///
    /// This is only useful if reconditioning and pointer support is enabled.
    #[clap(long)]
    pub skip_pointer_checks: bool,

    /// Logging configuration string (see https://docs.rs/tracing-subscriber/0.3.7/tracing_subscriber/struct.EnvFilter.html#directives)
    #[clap(long)]
    pub log: Option<String>,

    /// Minimum number of statements to generate in function bodies
    #[clap(long, default_value = "5")]
    pub fn_min_stmts: u32,

    /// Maximum number of statements to generate in function bodies
    #[clap(long, default_value = "5")]
    pub fn_max_stmts: u32,

    /// Minimum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, default_value = "0")]
    pub block_min_stmts: u32,

    /// Maximum number of statements to generate in blocks (if, loop, etc)
    #[clap(long, default_value = "5")]
    pub block_max_stmts: u32,

    /// Maximum nested block depth
    #[clap(long, default_value = "3")]
    pub max_block_depth: u32,

    /// Maximum number of function to generate
    #[clap(long, default_value = "5")]
    pub max_fns: u32,

    /// Minimum number of structs to generate (excluding input and output)
    #[clap(long, default_value = "1")]
    pub min_structs: u32,

    /// Maximum number of structs to generate (excluding input and output)
    #[clap(long, default_value = "5")]
    pub max_structs: u32,

    /// Minimum number of members allowed in a struct
    #[clap(long, default_value = "1")]
    pub min_struct_members: u32,

    /// Maximum number of members allowed in a struct
    #[clap(long, default_value = "5")]
    pub max_struct_members: u32,

    /// Preset options configuration. Individual options may still be overridden.
    #[clap(long)]
    pub preset: Option<Preset>,

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
