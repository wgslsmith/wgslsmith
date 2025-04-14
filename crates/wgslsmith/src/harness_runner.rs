use std::fmt::{Display, Write as _};
use std::io::{self, BufRead, BufReader, BufWriter, Write as _};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;

use eyre::eyre;
use harness_types::ConfigId;
use tap::Tap;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionResult {
    Success,
    Crash(String),
    Mismatch,
    // TODO: Detect timeouts from running harness
    // Might not actually be necessary since it's probably fine to treat them as successful runs
    // Timeout,
}

impl Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Success => write!(f, "success"),
            ExecutionResult::Crash(_) => write!(f, "crash"),
            ExecutionResult::Mismatch => write!(f, "mismatch"),
            // ExecutionResult::Timeout => write!(f, "timeout"),
        }
    }
}

pub enum Harness {
    Local(PathBuf),
    Remote(String),
}

pub fn exec_shader(
    harness: &Harness,
    config: Option<ConfigId>,
    shader: &str,
    metadata: &str,
    mut logger: impl FnMut(String),
) -> eyre::Result<ExecutionResult> {
    exec_shader_impl(harness, config, shader, metadata, &mut logger)
}

fn exec_shader_impl(
    harness: &Harness,
    config: Option<ConfigId>,
    shader: &str,
    metadata: &str,
    logger: &mut dyn FnMut(String),
) -> eyre::Result<ExecutionResult> {
    let mut cmd = match harness {
        Harness::Local(harness_path) => Command::new(harness_path).tap_mut(|cmd| {
            cmd.args(["run", "-", metadata]);
        }),
        Harness::Remote(remote) => Command::new(std::env::current_exe()?).tap_mut(|cmd| {
            cmd.args(["remote", remote, "run", "-", metadata]);
        }),
    };

    if let Some(config) = config {
        cmd.args(["-c", &config.to_string()]);
    }

    let mut harness = cmd
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

    let mut output = String::new();

    let status = wait_for_child_with_line_logger(harness, &mut |_, line| {
        writeln!(output, "{line}").unwrap();
        logger(line);
    })?;

    let result = match status.code() {
        None => return Err(eyre!("failed to get harness exit code")),
        Some(0) => ExecutionResult::Success,
        Some(1) => ExecutionResult::Mismatch,
        Some(101) => ExecutionResult::Crash(output),
        Some(code) => return Err(eyre!("harness exited with unrecognised code `{code}`")),
    };

    Ok(result)
}

#[derive(PartialEq, Eq)]
enum StdioKind {
    Stdout,
    Stderr,
}

fn wait_for_child_with_line_logger(
    mut child: Child,
    logger: &mut dyn FnMut(StdioKind, String),
) -> Result<ExitStatus, io::Error> {
    let (tx, rx) = crossbeam_channel::unbounded();

    child.stdout.take().map(|stdout| {
        thread::spawn({
            let tx = tx.clone();
            move || {
                BufReader::new(stdout)
                    .lines()
                    .map_while(Result::ok)
                    .try_for_each(|line| tx.send((StdioKind::Stdout, line)))
                    .unwrap();
            }
        })
    });

    child.stderr.take().map(|stderr| {
        thread::spawn({
            let tx = tx.clone();
            move || {
                BufReader::new(stderr)
                    .lines()
                    .map_while(Result::ok)
                    .try_for_each(|line| tx.send((StdioKind::Stderr, line)))
                    .unwrap();
            }
        })
    });

    drop(tx);

    while let Ok((kind, line)) = rx.recv() {
        logger(kind, line);
    }

    child.wait()
}
