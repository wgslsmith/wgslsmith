use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "reducer")]
use color_eyre::Help;
use directories::ProjectDirs;
#[cfg(feature = "reducer")]
use eyre::eyre;
use regex::Regex;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub harness: Harness,
    #[serde(default, rename = "remote")]
    pub remotes: HashMap<String, Remote>,
    #[serde(default)]
    pub fuzzer: Fuzzer,
    #[serde(default)]
    pub reducer: Reducer,
    #[serde(default)]
    pub validator: Validator,
}

#[derive(Default, Deserialize)]
pub struct Harness {
    pub path: Option<PathBuf>,
    pub remote: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct Remote {
    pub address: String,
}

#[derive(Default, Deserialize)]
pub struct Fuzzer {
    #[serde(with = "serde_regex")]
    pub ignore: Vec<Regex>,
}

#[derive(Default, Deserialize)]
pub struct Reducer {
    #[serde(default)]
    pub tmpdir: Option<String>,
    #[serde(default)]
    pub parallelism: Option<u32>,
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
    #[cfg(feature = "reducer")]
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or("creduce")
    }
}

#[derive(Default, Deserialize)]
pub struct Cvise {
    pub path: Option<String>,
}

impl Cvise {
    #[cfg(feature = "reducer")]
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or("cvise")
    }
}

#[derive(Default, Deserialize)]
pub struct Perses {
    pub jar: Option<String>,
}

impl Perses {
    #[cfg(feature = "reducer")]
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
    pub server: Option<String>,
}

impl Validator {
    #[cfg(feature = "reducer")]
    pub fn server(&self) -> eyre::Result<&str> {
        self.server.as_deref().ok_or_else(|| {
            eyre!("missing validation server address")
                .with_suggestion(|| "set `validator.server` in `wgslsmith.toml`")
        })
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> eyre::Result<Config> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                return match e.kind() {
                    io::ErrorKind::NotFound => Ok(Config::default()),
                    _ => Err(e.into()),
                }
            }
        };

        Ok(toml::from_slice(&bytes)?)
    }

    pub fn resolve_remote<'a>(&'a self, remote: &'a str) -> &'a str {
        if let Some(remote) = self.remotes.get(remote) {
            &remote.address
        } else {
            remote
        }
    }

    pub fn default_remote(&self) -> Option<&str> {
        self.harness
            .remote
            .as_deref()
            .map(|it| self.resolve_remote(it))
    }
}

pub fn default_path() -> eyre::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let project_dirs = ProjectDirs::from("", "", "wgslsmith");
    let config_dir = if let Some(dirs) = &project_dirs {
        dirs.config_dir()
    } else {
        exe.parent().unwrap()
    };
    Ok(config_dir.join("wgslsmith.toml"))
}
