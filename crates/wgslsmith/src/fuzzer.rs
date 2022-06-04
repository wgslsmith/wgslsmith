use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use clap::{ArgEnum, Parser};
use eyre::eyre;
use regex::Regex;
use tap::Tap;
use time::{format_description, OffsetDateTime};
use wait_timeout::ChildExt;

use crate::executor;

#[derive(Copy, Clone, ArgEnum)]
enum SaveStrategy {
    All,
    Crashes,
    Mismatches,
    /// Don't save any test cases - useful for debugging.
    Debug,
}

#[derive(Parser)]
pub struct Options {
    /// Path to directory in which to save failing test cases.
    #[clap(short, long, default_value = "out")]
    output: PathBuf,

    /// Strategy to use when determining which test cases to save.
    ///
    /// Note that `all` will still ignore crashes based on the `--ignore` option, if it is provided.
    #[clap(long, arg_enum, default_value = "all")]
    strategy: SaveStrategy,

    /// Regex for ignoring certain types of crashes.
    ///
    /// This will be matched against the stderr output from the test harness.
    #[clap(long)]
    ignore: Option<Regex>,

    /// Address of harness server.
    #[clap(short, long)]
    server: Option<String>,

    #[clap(long)]
    enable_pointers: bool,
}

enum Harness {
    Local,
    Remote(String),
}

fn gen_shader(options: &Options) -> eyre::Result<String> {
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("gen")
        .args(["--block-min-stmts", "1"])
        .args(["--block-max-stmts", "1"])
        .args(["--max-fns", "3"])
        .tap_mut(|cmd| {
            if options.enable_pointers {
                cmd.arg("--enable-pointers");
            }
        })
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(eyre!("wgslsmith command failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn recondition_shader(shader: &str) -> eyre::Result<String> {
    let mut reconditioner = Command::new(std::env::current_exe().unwrap())
        .arg("recondition")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = reconditioner.stdin.take().unwrap();
        let mut writer = BufWriter::new(stdin);
        write!(writer, "{shader}")?;
        writer.flush()?;
    }

    let output = reconditioner.wait_with_output()?;
    if !output.status.success() {
        return Err(eyre!("reconditioner command failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[derive(PartialEq, Eq)]
enum ExecutionResult {
    Success,
    Crash(String),
    Mismatch,
    Timeout,
}

impl ExecutionResult {
    fn should_save(&self, strategy: &SaveStrategy, ignore: Option<&Regex>) -> bool {
        match self {
            ExecutionResult::Success => false,
            ExecutionResult::Timeout => false,
            ExecutionResult::Crash(output) => {
                matches!(strategy, SaveStrategy::All | SaveStrategy::Crashes)
                    && !ignore.map(|it| it.is_match(output)).unwrap_or(false)
            }
            ExecutionResult::Mismatch => {
                matches!(strategy, SaveStrategy::All | SaveStrategy::Mismatches)
            }
        }
    }
}

impl Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Success => write!(f, "success"),
            ExecutionResult::Crash(_) => write!(f, "crash"),
            ExecutionResult::Mismatch => write!(f, "mismatch"),
            ExecutionResult::Timeout => write!(f, "timeout"),
        }
    }
}

fn exec_shader(harness: &Harness, shader: &str, metadata: &str) -> eyre::Result<ExecutionResult> {
    match harness {
        Harness::Local => {
            let mut harness = Command::new(std::env::current_exe().unwrap())
                .arg("harness")
                .args(["run", "-", metadata])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()?;

            {
                let stdin = harness.stdin.take().unwrap();
                let mut writer = BufWriter::new(stdin);
                write!(writer, "{shader}")?;
                writer.flush()?;
            }

            let mut stdout = harness.stdout.take().unwrap();
            let stderr = harness.stderr.take().unwrap();

            std::thread::spawn(move || {
                std::io::copy(&mut stdout, &mut std::io::stdout()).unwrap();
            });

            let stderr_thread = std::thread::spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut output = String::new();
                let mut buffer = String::new();

                while let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 {
                        break;
                    }

                    eprint!("{buffer}");

                    output += &buffer;
                    buffer.clear();
                }

                output
            });

            let result = harness.wait_timeout(Duration::from_secs(60))?;

            let status = match result {
                Some(status) => status,
                None => {
                    harness.kill()?;
                    return Ok(ExecutionResult::Timeout);
                }
            };

            let stderr = stderr_thread.join().unwrap();
            let result = match status.code() {
                None => return Err(eyre!("failed to get harness exit code")),
                Some(0) => ExecutionResult::Success,
                Some(1) => ExecutionResult::Mismatch,
                Some(101) => ExecutionResult::Crash(stderr),
                Some(code) => return Err(eyre!("harness exited with unrecognised code `{code}`")),
            };

            Ok(result)
        }
        Harness::Remote(address) => {
            let response = executor::exec_shader(address, shader, metadata)?;
            let result = match response.exit_code {
                0 => ExecutionResult::Success,
                1 => ExecutionResult::Mismatch,
                101 => ExecutionResult::Crash(response.output),
                code => return Err(eyre!("harness exited with unrecognised code `{code}`")),
            };

            Ok(result)
        }
    }
}

fn save_shader(out: &Path, shader: &str, reconditioned: &str, metadata: &str) -> eyre::Result<()> {
    let timestamp = OffsetDateTime::now_local()?.format(&format_description::parse(
        "[year]-[month]-[day]-[hour]-[minute]-[second]",
    )?)?;

    let out = out.join(&timestamp);

    std::fs::create_dir_all(&out)?;

    std::fs::write(out.join("shader.wgsl"), shader)?;
    std::fs::write(out.join("reconditioned.wgsl"), reconditioned)?;
    std::fs::write(out.join("inputs.json"), metadata)?;

    Ok(())
}

pub fn run(options: Options) -> eyre::Result<()> {
    let harness = if let Some(server) = &options.server {
        Harness::Remote(server.clone())
    } else {
        Harness::Local
    };

    loop {
        let shader = gen_shader(&options)?;
        let (metadata, shader) = shader
            .split_once('\n')
            .ok_or_else(|| eyre!("expected first line of shader to be a JSON metadata comment"))?;

        let metadata = metadata.trim_start_matches("//").trim();
        let reconditioned = match recondition_shader(shader) {
            Ok(reconditioned) => reconditioned,
            Err(_) => {
                eprintln!("reconditioner command failed, ignoring");
                continue;
            }
        };

        let result = match exec_shader(&harness, &reconditioned, metadata) {
            Ok(result) => result,
            Err(e) => {
                save_shader(&options.output, shader, &reconditioned, metadata)?;
                return Err(e);
            }
        };

        if result == ExecutionResult::Timeout {
            eprintln!("warning: shader execution timed out");
        } else if result.should_save(&options.strategy, options.ignore.as_ref()) {
            save_shader(&options.output, shader, &reconditioned, metadata)?;
        }
    }
}
