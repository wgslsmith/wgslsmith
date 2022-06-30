use std::ffi::OsStr;
use std::fs::Permissions;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, thread};

use clap::{ArgEnum, Parser};
use eyre::{eyre, Context};
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use regex::Regex;
use signal_hook::consts::{SIGUSR1, SIGUSR2};
use tap::Tap;

use crate::compiler::{Backend, Compiler};
use crate::config::Config;

#[derive(ArgEnum, Clone)]
pub enum ReductionKind {
    Crash,
    Mismatch,
}

#[derive(Parser)]
pub struct Options {
    /// Type of bug that is being reduced.
    #[clap(arg_enum)]
    kind: ReductionKind,

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

    /// This passed to the underlying reducer using the appropriate flag, to set how many threads it
    /// should use.
    ///
    /// Can also be set in `wgslsmith.toml`, as `reducer.parallelism`.
    #[clap(long)]
    parallelism: Option<u32>,
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
        threads: u32,
        shader: impl AsRef<OsStr>,
        test: impl AsRef<OsStr>,
    ) -> eyre::Result<Command> {
        fn build_creduce(
            path: &str,
            shader: impl AsRef<OsStr>,
            test: impl AsRef<OsStr>,
            threads: u32,
        ) -> Command {
            let mut cmd = Command::new(path);
            cmd.arg(test);
            cmd.arg(shader);
            cmd.arg("--not-c");
            cmd.arg("--n").arg(threads.to_string());
            cmd
        }

        match self {
            Reducer::Creduce => Ok(build_creduce(
                config.reducer.creduce.path(),
                shader,
                test,
                threads,
            )),
            Reducer::Cvise => Ok(build_creduce(
                config.reducer.cvise.path(),
                shader,
                test,
                threads,
            )),
            Reducer::Perses => {
                let perses_jar = config.reducer.perses.jar()?;

                Ok(Command::new("java").tap_mut(|cmd| {
                    cmd.args(["-jar", perses_jar])
                        .arg("-i")
                        .arg(shader)
                        .arg("-t")
                        .arg(test)
                        .arg("-o")
                        .arg(".")
                        .arg("--threads")
                        .arg(threads.to_string());
                }))
            }
            Reducer::Picire => Ok(Command::new("picire").tap_mut(|cmd| {
                cmd.arg("-i")
                    .arg(shader)
                    .arg("--test")
                    .arg(test)
                    .arg("--parallel")
                    .args(["-o", "."])
                    .arg("-j")
                    .arg(threads.to_string());
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

pub fn run(config: Config, options: Options) -> eyre::Result<()> {
    let pid = std::process::id();
    std::env::set_var("WGSLREDUCE_PID", pid.to_string());

    let worker = thread::spawn(move || {
        let result = thread_main(&config, options);
        nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGUSR2).unwrap();
        result
    });

    let mut count = 0;

    for signal in &mut signal_hook::iterator::Signals::new([SIGUSR1, SIGUSR2]).unwrap() {
        if signal == SIGUSR1 {
            count += 1;
        } else if signal == SIGUSR2 {
            worker.join().unwrap()?;
            break;
        }
    }

    println!("> {count} calls to interestingness test");

    Ok(())
}

fn thread_main(config: &Config, options: Options) -> eyre::Result<()> {
    let shader_path = Path::new(&options.shader);
    if !shader_path.exists() {
        return Err(eyre!("shader at {shader_path:?} does not exist"));
    }

    let shader_path = shader_path.canonicalize()?;

    let input_path = if let Some(input_path) = options.input_data {
        input_path
    } else {
        let mut try_path = shader_path
            .parent()
            .unwrap()
            .join(shader_path.file_stem().unwrap())
            .with_extension("json");

        if !try_path.exists() {
            try_path = shader_path.parent().unwrap().join("inputs.json");
        }

        if !try_path.exists() {
            return Err(eyre!(
                "couldn't determine path to inputs file, pass one explicitly"
            ));
        }

        try_path
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

    let parallelism = options
        .parallelism
        .or(config.reducer.parallelism)
        .unwrap_or(1);

    let mut cmd = reducer
        .cmd(config, parallelism, shader_name, "test.sh")?
        .tap_mut(|cmd| {
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
        ReductionKind::Crash => {
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
        ReductionKind::Mismatch => {
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
    // let reconditioned_path = out_dir
    //     .join("reconditioned.wgsl")
    //     .to_str()
    //     .unwrap()
    //     .to_owned();

    crate::fmt::run(crate::fmt::Options {
        input: result_path.clone(),
        output: result_path,
    })?;

    // crate::reconditioner::run(crate::reconditioner::Options {
    //     input: result_path,
    //     output: reconditioned_path,
    // })?;

    Ok(())
}

fn setup_out_dir(out_dir: &Path, shader: &Path, reducer: &Reducer) -> eyre::Result<()> {
    // Create output dir
    if !out_dir.exists() {
        std::fs::create_dir(out_dir)
            .wrap_err_with(|| eyre!("failed to create dir `{}`", out_dir.display()))?;
    } else if std::fs::read_dir(out_dir)?.next().is_some() {
        return Err(eyre!("`{}` is not empty", out_dir.display()));
    }

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
