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
    #[serde(default)]
    pub tmpdir: Option<String>,
    #[serde(default)]
    pub creduce: Creduce,
    #[serde(default)]
    pub cvise: Cvise,
    #[serde(default)]
    pub perses: Perses,
}

#[derive(Default, Deserialize)]
pub struct Creduce {
    pub path: Option<String>,
}

impl Creduce {
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or("creduce")
    }
}

#[derive(Default, Deserialize)]
pub struct Cvise {
    pub path: Option<String>,
}

impl Cvise {
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or("cvise")
    }
}

#[derive(Default, Deserialize)]
pub struct Perses {
    pub jar: Option<String>,
}

impl Perses {
    pub fn jar(&self) -> eyre::Result<&str> {
        self.jar.as_deref().ok_or_else(|| {
            eyre!("missing path to perses jar file")
                .with_suggestion(|| "set `reducer.perses.jar` in `wgslsmith.toml`")
        })
    }
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
    pub server: Option<String>,
}

impl Fxc {
    pub fn server(&self) -> eyre::Result<&str> {
        self.server.as_deref().ok_or_else(|| {
            eyre!("missing fxc server address")
                .with_suggestion(|| "set `validator.fxc.server` in `wgslsmith.toml`")
        })
    }
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
