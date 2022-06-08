use std::env;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::Permissions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use clap::{ArgEnum, Parser};
use eyre::eyre;
use regex::Regex;
use tap::Tap;

use crate::config::Config;

#[derive(ArgEnum, Clone)]
pub enum Kind {
    Crash,
    Mismatch,
}

#[derive(ArgEnum, Clone)]
pub enum Compiler {
    Tint,
    Naga,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Compiler::Tint => "tint",
            Compiler::Naga => "naga",
        };

        write!(f, "{val}")
    }
}

#[derive(ArgEnum, Clone, Copy)]
pub enum Backend {
    Hlsl,
    Msl,
    Spirv,
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Backend::Hlsl => "hlsl",
            Backend::Msl => "msl",
            Backend::Spirv => "spirv",
        };

        write!(f, "{val}")
    }
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
    #[clap(long, conflicts_with("compiler"))]
    config: Option<String>,

    /// Compiler to use for reducing a crash.
    #[clap(long, arg_enum, requires("backend"))]
    compiler: Option<Compiler>,

    /// Compiler backend to use for reducing a crash.
    #[clap(long, arg_enum)]
    backend: Option<Backend>,

    /// Regex to match crash output against.
    ///
    /// This is only valid if we're reducing a crash.
    #[clap(long, required_if_eq("kind", "crash"))]
    regex: Option<Regex>,

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
    Cvise,
    Perses,
    Picire,
}

impl Reducer {
    fn cmd(
        &self,
        config: &Config,
        shader: impl AsRef<OsStr>,
        test: impl AsRef<OsStr>,
    ) -> eyre::Result<Command> {
        fn build_creduce(
            path: &str,
            shader: impl AsRef<OsStr>,
            test: impl AsRef<OsStr>,
        ) -> Command {
            let mut cmd = Command::new(path);
            cmd.arg(test);
            cmd.arg(shader);
            cmd.arg("--not-c");
            cmd
        }

        match self {
            Reducer::Creduce => Ok(build_creduce(config.reducer.creduce.path(), shader, test)),
            Reducer::Cvise => Ok(build_creduce(config.reducer.cvise.path(), shader, test)),
            Reducer::Perses => {
                let perses_jar = config.reducer.perses.jar()?;

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
            Reducer::Picire => Ok(Command::new("picire").tap_mut(|cmd| {
                cmd.arg("-i")
                    .arg(shader)
                    .arg("--test")
                    .arg(test)
                    .arg("--parallel")
                    .args(["-j", "24"]);
            })),
        }
    }

    fn gen_test_script(&self) -> String {
        let exe = env::current_exe().unwrap();
        let template = match self {
            Reducer::Picire => include_str!("test-picire.sh"),
            _ => include_str!("test.sh"),
        };
        template.replacen("[WGSLSMITH]", exe.to_str().unwrap(), 1)
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

    let reducer = options.reducer.unwrap_or_else(|| {
        if config.reducer.perses.jar.is_some() {
            Reducer::Perses
        } else {
            Reducer::Creduce
        }
    });

    println!("> using reducer: {reducer:?}");

    setup_out_dir(&out_dir, &options.shader, &reducer)?;

    let harness_server = options
        .server
        .as_deref()
        .or(config.harness.server.as_deref());

    let mut cmd = reducer.cmd(config, shader_name, "test.sh")?.tap_mut(|cmd| {
        cmd.current_dir(&out_dir)
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
            cmd.env("WGSLREDUCE_KIND", "crash")
                .env("WGSLREDUCE_REGEX", options.regex.unwrap().as_str());

            if let Some(config) = options.config {
                cmd.env("WGSLREDUCE_CONFIG", config);
            } else {
                let compiler = options.compiler.unwrap();
                let backend = options.backend.unwrap();
                cmd.env("WGSLREDUCE_COMPILER", compiler.to_string())
                    .env("WGSLREDUCE_BACKEND", backend.to_string());
            }

            if !options.no_recondition {
                cmd.env("WGSLREDUCE_RECONDITION", "1");
            }
        }
        Kind::Mismatch => {
            cmd.env("WGSLREDUCE_KIND", "mismatch");
        }
    }

    let start_time = Instant::now();

    if !cmd.status()?.success() {
        return Err(eyre!("reducer process did not exit successfully"));
    }

    let end_time = Instant::now();
    let duration = end_time - start_time;

    println!("> reducer completed in {}s", duration.as_secs_f64());

    let result_path = out_dir.join(shader_name).to_str().unwrap().to_owned();

    crate::fmt::run(crate::fmt::Options {
        input: result_path.clone(),
        output: result_path,
    })?;

    Ok(())
}

fn setup_out_dir(out_dir: &Path, shader: &Path, reducer: &Reducer) -> eyre::Result<()> {
    // Create output dir
    std::fs::create_dir(out_dir)?;

    // Copy over the shader file
    std::fs::copy(shader, out_dir.join(shader.file_name().unwrap()))?;

    // Generate the interestingness test script
    let test_path = out_dir.join("test.sh");
    std::fs::write(&test_path, reducer.gen_test_script())?;

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        // Make sure the test script is executable
        std::fs::set_permissions(test_path, Permissions::from_mode(0o755))?;
    }

    Ok(())
}
