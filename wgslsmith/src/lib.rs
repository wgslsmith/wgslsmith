use std::hash::BuildHasher;

use hashers::fx_hash::FxHasher;

mod macros;

pub mod generator;
pub mod types;

#[derive(Clone)]
struct BuildFxHasher;

impl BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::default()
    }
}
