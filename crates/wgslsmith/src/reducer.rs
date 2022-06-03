use std::env;
use std::ffi::OsStr;
use std::fs::Permissions;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{ArgEnum, Parser};
use color_eyre::Help;
use eyre::eyre;
use regex::Regex;
use tap::Tap;

use crate::config::Config;

#[derive(ArgEnum, Clone)]
enum Kind {
    Crash,
    Mismatch,
}

#[derive(Parser)]
pub struct Options {
    /// Type of bug that is being reduced.
    #[clap(arg_enum)]
    kind: Kind,

    /// Path to the WGSL shader file to reduce.
    shader: PathBuf,

    /// Path to the input data file.
    ///
    /// If not set, the program will look for a JSON file with the same name as the shader.
    input_data: Option<PathBuf>,

    /// Path to output directory for reduced shader.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Address of harness server.
    #[clap(short, long)]
    server: Option<String>,

    /// Config to use for reducing a crash.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(short, long)]
    config: Option<String>,

    /// Regex to match crash output against.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(long, default_value = "")]
    regex: Regex,

    /// Don't recondition shader before executing.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(long)]
    no_recondition: bool,

    /// Disable logging from harness.
    #[clap(short, long)]
    quiet: bool,

    #[clap(long, arg_enum)]
    reducer: Option<Reducer>,
}

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum Reducer {
    Creduce,
    Perses,
}

impl Reducer {
    fn cmd(
        &self,
        config: &Config,
        shader: impl AsRef<OsStr>,
        test: impl AsRef<OsStr>,
    ) -> eyre::Result<Command> {
        match self {
            Reducer::Creduce => Ok(Command::new("creduce").tap_mut(|cmd| {
                cmd.arg(test).arg(shader).arg("--not-c");
            })),
            Reducer::Perses => {
                let perses_jar = config.reducer.perses.jar.as_deref().ok_or_else(|| {
                    eyre!("missing path to perses jar file")
                        .with_suggestion(|| "set `reducer.perses.jar` in `wgslsmith.toml`")
                })?;

                Ok(Command::new("java").tap_mut(|cmd| {
                    cmd.args(["-jar", perses_jar])
                        .arg("-i")
                        .arg(shader)
                        .arg("-t")
                        .arg(test)
                        .arg("-o")
                        .arg(".");
                }))
            }
        }
    }
}

pub fn run(config: &Config, options: Options) -> eyre::Result<()> {
    let shader_path = Path::new(&options.shader);
    if !shader_path.exists() {
        return Err(eyre!("shader at {shader_path:?} does not exist"));
    }

    let shader_path = shader_path.canonicalize()?;

    let input_path = if let Some(input_path) = options.input_data {
        input_path
    } else {
        shader_path
            .parent()
            .unwrap()
            .join(shader_path.file_stem().unwrap())
            .with_extension("json")
    };

    if !input_path.exists() {
        return Err(eyre!("file at {input_path:?} does not exist"));
    }

    let metadata_path = input_path.canonicalize()?;

    let out_dir = options.output.unwrap_or_else(|| {
        let out_dir = options.shader.parent().unwrap().join("reduced");
        if out_dir.exists() {
            let mut n = 1;
            loop {
                let path = out_dir.with_file_name(format!("reduced-{n}"));
                if !path.exists() {
                    break path;
                }
                n += 1
            }
        } else {
            out_dir
        }
    });

    let shader_name = options.shader.file_name().unwrap();

    setup_out_dir(&out_dir, &options.shader)?;

    let reducer = options.reducer.unwrap_or_else(|| {
        if config.reducer.perses.jar.is_some() {
            Reducer::Perses
        } else {
            Reducer::Creduce
        }
    });

    println!("> using reducer: {reducer:?}");

    let harness_server = options
        .server
        .as_deref()
        .or(config.harness.server.as_deref());

    let mut cmd = reducer.cmd(config, shader_name, "test.sh")?.tap_mut(|cmd| {
        cmd.current_dir(out_dir)
            .env("WGSLREDUCE_SHADER_NAME", shader_path.file_name().unwrap())
            .env("WGSLREDUCE_METADATA_PATH", metadata_path);

        if let Some(server) = harness_server {
            cmd.env("WGSLREDUCE_SERVER", server);
        }

        if let Some(tmpdir) = &config.reducer.tmpdir {
            cmd.env("TMPDIR", tmpdir);
        }
    });

    match options.kind {
        Kind::Crash => {
            let config = match options.config {
                Some(config) => config,
                None => return Err(eyre!("a configuration is required when reducing a crash")),
            };

            cmd.env("WGSLREDUCE_KIND", "crash")
                .env("WGSLREDUCE_CONFIGURATIONS", config)
                .env("WGSLREDUCE_REGEX", options.regex.as_str());

            if !options.no_recondition {
                cmd.env("WGSLREDUCE_RECONDITION", "1");
            }
        }
        Kind::Mismatch => {
            cmd.env("WGSLREDUCE_KIND", "mismatch");
        }
    }

    if !cmd.status()?.success() {
        return Err(eyre!("reducer process did not exit successfully"));
    }

    Ok(())
}

fn setup_out_dir(out_dir: &Path, shader: &Path) -> eyre::Result<()> {
    // Create output dir
    std::fs::create_dir(out_dir)?;

    // Copy over the shader file
    std::fs::copy(shader, out_dir.join(shader.file_name().unwrap()))?;

    // Generate the interestingness test script
    let test_path = out_dir.join("test.sh");
    let test_script = format!(
        "#!/usr/bin/env bash\n\"{}\" test\n",
        env::current_exe().unwrap().display()
    );
    std::fs::write(&test_path, test_script)?;

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        // Make sure the test script is executable
        std::fs::set_permissions(test_path, Permissions::from_mode(0o755))?;
    }

    Ok(())
}
