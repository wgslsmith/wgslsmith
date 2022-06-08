use std::path::Path;

use color_eyre::Help;
use eyre::eyre;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub harness: Harness,
    #[serde(default)]
    pub reducer: Reducer,
    #[serde(default)]
    pub validator: Validator,
}

#[derive(Default, Deserialize)]
pub struct Harness {
    pub target: Option<String>,
    pub server: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct Reducer {
    pub tmpdir: Option<String>,
    #[serde(default)]
    pub perses: Perses,
}

#[derive(Default, Deserialize)]
pub struct Perses {
    pub jar: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct Validator {
    #[serde(default)]
    pub fxc: Fxc,
    #[serde(default)]
    pub metal: Metal,
}

#[derive(Default, Deserialize)]
pub struct Fxc {
    #[serde(default)]
    pub use_wine: bool,
}

#[derive(Default, Deserialize)]
pub struct Metal {
    #[serde(default)]
    pub path: Option<String>,
}

impl Metal {
    pub fn path(&self) -> eyre::Result<&str> {
        self.path.as_deref().ok_or_else(|| {
            eyre!("missing path to metal compiler")
                .with_suggestion(|| "set `validator.metal.path` in `wgslsmith.toml`")
        })
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> eyre::Result<Config> {
        toml::from_slice(&std::fs::read(path)?).map_err(Into::into)
    }
}
