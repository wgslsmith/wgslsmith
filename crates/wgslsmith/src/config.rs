use std::path::Path;

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub harness: Harness,
    #[serde(default)]
    pub reducer: Reducer,
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

impl Config {
    pub fn load(path: impl AsRef<Path>) -> eyre::Result<Config> {
        toml::from_slice(&std::fs::read(path)?).map_err(Into::into)
    }
}
