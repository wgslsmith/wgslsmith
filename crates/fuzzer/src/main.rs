use std::env;
use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::anyhow;
use clap::{ArgEnum, Parser};
use regex::Regex;
use time::{format_description, OffsetDateTime};
use wait_timeout::ChildExt;

#[derive(Copy, Clone, ArgEnum)]
enum SaveStrategy {
    All,
    Crashes,
    Mismatches,
    /// Don't save any test cases - useful for debugging.
    Debug,
}

#[derive(Parser)]
struct Options {
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
}

struct Tools {
    wgslsmith: PathBuf,
    reconditioner: PathBuf,
    harness: PathBuf,
}

impl Tools {
    fn find() -> anyhow::Result<Tools> {
        let current_exe = std::env::current_exe()?;
        let bin_dir = current_exe.parent().unwrap();

        let harness_path = env::var("HARNESS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // First try to find the harness next to the current executable
                let harness_path = bin_dir.join("harness");
                if harness_path.exists() {
                    return harness_path;
                }

                let harness_path = bin_dir.join("harness.exe");
                if harness_path.exists() {
                    return harness_path;
                }

                // If that fails and the current executable looks like it's in the target dir,
                // try searching using the known project structure
                if bin_dir.ends_with("target/release") {
                    let project_dir = bin_dir.parent().unwrap().parent().unwrap();

                    // First try searching the default harness target dir
                    let harness_target_dir = project_dir.join("harness/target/release");

                    let harness_path = harness_target_dir.join("harness");
                    if harness_path.exists() {
                        return harness_path;
                    }

                    let harness_path = harness_target_dir.join("harness.exe");
                    if harness_path.exists() {
                        return harness_path;
                    }

                    // Then try searching the windows-msvc target dir, in case we're cross compiling
                    // to windows
                    let harness_target_dir =
                        project_dir.join("harness/target/x86_64-pc-windows-msvc/release");

                    let harness_path = harness_target_dir.join("harness.exe");
                    if harness_path.exists() {
                        return harness_path;
                    }
                }

                PathBuf::from("harness")
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

fn exec_shader(tools: &Tools, shader: &str, metadata: &str) -> anyhow::Result<ExecutionResult> {
    let mut harness = Command::new(&tools.harness)
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
        None => return Err(anyhow!("failed to get harness exit code")),
        Some(0) => ExecutionResult::Success,
        Some(1) => ExecutionResult::Mismatch,
        Some(101) => ExecutionResult::Crash(stderr),
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

    let out = out.join(&timestamp);

    std::fs::create_dir_all(&out)?;

    std::fs::write(out.join("shader.wgsl"), shader)?;
    std::fs::write(out.join("reconditioned.wgsl"), reconditioned)?;
    std::fs::write(out.join("meta.json"), metadata)?;

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

        let result = match exec_shader(&tools, &reconditioned, metadata) {
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
