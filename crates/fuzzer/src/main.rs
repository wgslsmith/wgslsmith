use std::env;
use std::fmt::Display;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::anyhow;
use clap::{ArgEnum, Parser};
use time::{format_description, OffsetDateTime};
use wait_timeout::ChildExt;

#[derive(Copy, Clone, ArgEnum)]
enum SaveStrategy {
    All,
    Crashes,
    Mismatches,
}

impl SaveStrategy {
    fn should_save(&self, result: &ExecutionResult) -> bool {
        match self {
            SaveStrategy::All => {
                matches!(result, ExecutionResult::Crash | ExecutionResult::Mismatch)
            }
            SaveStrategy::Crashes => matches!(result, ExecutionResult::Crash),
            SaveStrategy::Mismatches => matches!(result, ExecutionResult::Mismatch),
        }
    }
}

#[derive(Parser)]
struct Options {
    /// Path to directory in which to save failing test cases.
    #[clap(short, long, default_value = "out")]
    output: PathBuf,

    /// Strategy to use when determining which test cases to save.
    #[clap(long, arg_enum, default_value = "all")]
    strategy: SaveStrategy,
}

struct Tools {
    wgslsmith: PathBuf,
    reconditioner: PathBuf,
    harness: PathBuf,
}

impl Tools {
    fn find() -> anyhow::Result<Tools> {
        let script_dir = PathBuf::from(env::var("SCRIPT_DIR")?);
        let project_dir = script_dir.parent().unwrap();
        let bin_dir = project_dir.join("target/release");

        let harness_path = env::var("HARNESS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(target_os = "windows")]
                let harness_bin = "harness.exe";
                #[cfg(not(target_os = "windows"))]
                let harness_bin = "harness";
                project_dir.join("harness/target/release").join(harness_bin)
            });

        let tools = Tools {
            wgslsmith: bin_dir.join("wgslsmith"),
            reconditioner: bin_dir.join("reconditioner"),
            harness: harness_path,
        };

        fn ensure(name: &str, path: &Path) -> anyhow::Result<()> {
            if !path.exists() {
                return Err(anyhow!(
                    "couldn't find executable for `{name}` at `{}`",
                    path.display()
                ));
            } else {
                Ok(())
            }
        }

        ensure("wgslsmith", &tools.wgslsmith)?;
        ensure("reconditioner", &tools.reconditioner)?;
        ensure("harness", &tools.harness)?;

        Ok(tools)
    }

    fn print(&self) {
        fn print(name: &str, path: &Path) {
            println!("\t{:16} : {}", name, path.display());
        }

        println!("Detected tools paths:");
        print("wgslsmith", &self.wgslsmith);
        print("reconditioner", &self.reconditioner);
        print("harness", &self.harness);
    }
}

fn gen_shader(tools: &Tools) -> anyhow::Result<String> {
    let output = Command::new(&tools.wgslsmith)
        .args(["--block-min-stmts", "1"])
        .args(["--block-max-stmts", "1"])
        .args(["--max-fns", "3"])
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("wgslsmith command failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn recondition_shader(tools: &Tools, shader: &str) -> anyhow::Result<String> {
    let mut reconditioner = Command::new(&tools.reconditioner)
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
        return Err(anyhow!("reconditioner command failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[derive(PartialEq, Eq)]
enum ExecutionResult {
    Success,
    Crash,
    Mismatch,
    Timeout,
}

impl Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Success => write!(f, "success"),
            ExecutionResult::Crash => write!(f, "crash"),
            ExecutionResult::Mismatch => write!(f, "mismatch"),
            ExecutionResult::Timeout => write!(f, "timeout"),
        }
    }
}

fn exec_shader(tools: &Tools, shader: &str, metadata: &str) -> anyhow::Result<ExecutionResult> {
    let mut harness = Command::new(&tools.harness)
        .args(["--metadata", metadata])
        .stdin(Stdio::piped())
        .spawn()?;

    {
        let stdin = harness.stdin.take().unwrap();
        let mut writer = BufWriter::new(stdin);
        write!(writer, "{shader}")?;
        writer.flush()?;
    }

    let result = harness.wait_timeout(Duration::from_secs(60))?;

    let status = match result {
        Some(status) => status,
        None => {
            harness.kill()?;
            return Ok(ExecutionResult::Timeout);
        }
    };

    let result = match status.code() {
        None => return Err(anyhow!("failed to get harness exit code")),
        Some(0) => ExecutionResult::Success,
        Some(1) => ExecutionResult::Mismatch,
        Some(101) => ExecutionResult::Crash,
        Some(code) => return Err(anyhow!("harness exited with unrecognised code `{code}`")),
    };

    Ok(result)
}

fn save_shader(
    out: &Path,
    shader: &str,
    reconditioned: &str,
    metadata: &str,
) -> anyhow::Result<()> {
    let timestamp = OffsetDateTime::now_local()?.format(&format_description::parse(
        "[year]-[month]-[day]-[hour]-[minute]-[second]",
    )?)?;

    std::fs::create_dir_all(out)?;

    std::fs::write(out.join(&timestamp).with_extension("wgsl"), shader)?;
    std::fs::write(
        out.join(&timestamp).with_extension("reconditioned.wgsl"),
        reconditioned,
    )?;
    std::fs::write(out.join(&timestamp).with_extension("json"), metadata)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();
    let tools = Tools::find()?;

    tools.print();

    loop {
        let shader = gen_shader(&tools)?;
        let (metadata, shader) = shader.split_once('\n').ok_or_else(|| {
            anyhow!("expected first line of shader to be a JSON metadata comment")
        })?;

        let metadata = metadata.trim_start_matches("//").trim();
        let reconditioned = recondition_shader(&tools, shader)?;

        let result = exec_shader(&tools, &reconditioned, metadata)?;

        if result == ExecutionResult::Timeout {
            eprintln!("warning: shader execution timed out");
        } else if options.strategy.should_save(&result) {
            save_shader(&options.output, shader, &reconditioned, metadata)?;
        }
    }
}
