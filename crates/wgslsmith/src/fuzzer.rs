use std::fmt::Display;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::{Parser, ValueEnum};
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use eyre::eyre;
use regex::Regex;
use tap::Tap;
use time::{format_description, OffsetDateTime, UtcOffset};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::Rect;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;
use wait_timeout::ChildExt;

use crate::config::Config;
use crate::executor;

#[derive(Copy, Clone, ValueEnum)]
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
    #[clap(short, long, action, default_value = "out")]
    output: PathBuf,

    /// Strategy to use when determining which test cases to save.
    ///
    /// Note that `all` will still ignore crashes based on the `--ignore` option, if it is provided.
    #[clap(long, action, action, default_value = "all")]
    strategy: SaveStrategy,

    /// Regex for ignoring certain types of crashes.
    ///
    /// This will be matched against the stderr output from the test harness.
    #[clap(long, action)]
    ignore: Vec<Regex>,

    /// Address of harness server.
    #[clap(short, long, action)]
    server: Option<String>,

    #[clap(long, action)]
    enable_pointers: bool,

    /// Specific harness configuration to test.
    #[clap(long, action)]
    config: Option<String>,
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
    fn should_save<'a>(
        &self,
        strategy: &SaveStrategy,
        mut ignore: impl Iterator<Item = &'a Regex>,
    ) -> bool {
        match self {
            ExecutionResult::Success => false,
            ExecutionResult::Timeout => false,
            ExecutionResult::Crash(output) => {
                matches!(strategy, SaveStrategy::All | SaveStrategy::Crashes)
                    && !ignore.any(|it| it.is_match(output))
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

fn exec_shader(
    harness: &Harness,
    options: &Options,
    shader: &str,
    metadata: &str,
) -> eyre::Result<ExecutionResult> {
    match harness {
        Harness::Local => {
            let mut harness = Command::new(std::env::current_exe().unwrap())
                .args(["run", "-", metadata])
                .tap_mut(|cmd| {
                    if let Some(config) = &options.config {
                        cmd.args(["-c", config]);
                    }
                })
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()?;

            {
                let stdin = harness.stdin.take().unwrap();
                let mut writer = BufWriter::new(stdin);
                write!(writer, "{shader}")?;
                writer.flush()?;
            }

            // let mut stdout = harness.stdout.take().unwrap();
            let stderr = harness.stderr.take().unwrap();

            // std::thread::spawn(move || {
            //     std::io::copy(&mut stdout, &mut std::io::stdout()).unwrap();
            // });

            let stderr_thread = std::thread::spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut output = String::new();
                let mut buffer = String::new();

                while let Ok(bytes) = reader.read_line(&mut buffer) {
                    if bytes == 0 {
                        break;
                    }

                    // eprint!("{buffer}");

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
            let response = executor::exec_shader(address, shader.to_owned(), metadata.to_owned())?;
            let result = match response.exit_code {
                0 => ExecutionResult::Success,
                1 => ExecutionResult::Mismatch,
                2 => ExecutionResult::Timeout,
                101 => ExecutionResult::Crash(response.output),
                code => return Err(eyre!("harness exited with unrecognised code `{code}`")),
            };

            Ok(result)
        }
    }
}

static mut UTC_OFFSET: Option<UtcOffset> = None;

fn save_shader(
    out: &Path,
    shader: &str,
    reconditioned: &str,
    metadata: &str,
    output: Option<&str>,
) -> eyre::Result<()> {
    let now = OffsetDateTime::now_utc().to_offset(unsafe { UTC_OFFSET }.unwrap());
    let timestamp = now.format(&format_description::parse(
        "[year]-[month]-[day]-[hour]-[minute]-[second]",
    )?)?;

    let out = out.join(&timestamp);

    std::fs::create_dir_all(&out)?;

    std::fs::write(out.join("shader.wgsl"), shader)?;
    std::fs::write(out.join("reconditioned.wgsl"), reconditioned)?;
    std::fs::write(out.join("inputs.json"), metadata)?;

    if let Some(output) = output {
        std::fs::write(out.join("stderr.txt"), output.replace('\0', ""))?;
    }

    Ok(())
}

pub fn run(config: Config, options: Options) -> eyre::Result<()> {
    unsafe { UTC_OFFSET = Some(UtcOffset::current_local_offset()?) };

    let harness = if let Some(server) = options
        .server
        .as_deref()
        .or_else(|| config.default_remote())
    {
        Harness::Remote(server.to_owned())
    } else {
        Harness::Local
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let ui = Arc::new(Mutex::new(Ui::new(terminal, UiState::default())));

    let worker = {
        let ui = ui.clone();
        move || -> eyre::Result<()> {
            loop {
                let shader = gen_shader(&options)?;
                let (metadata, shader) = shader.split_once('\n').ok_or_else(|| {
                    eyre!("expected first line of shader to be a JSON metadata comment")
                })?;

                let metadata = metadata.trim_start_matches("//").trim();
                let reconditioned = match recondition_shader(shader) {
                    Ok(reconditioned) => reconditioned,
                    Err(_) => {
                        eprintln!("reconditioner command failed, ignoring");
                        continue;
                    }
                };

                let exec_result = exec_shader(&harness, &options, &reconditioned, metadata);

                let mut ui = ui.lock().unwrap();

                ui.state.total += 1;

                let result = match exec_result {
                    Ok(result) => result,
                    Err(_) => {
                        ui.state.failures += 1;
                        // save_shader(&options.output, shader, &reconditioned, metadata)?;
                        // return Err(e);
                        continue;
                    }
                };

                match result {
                    ExecutionResult::Success => ui.state.success += 1,
                    ExecutionResult::Crash(_) => ui.state.crashes += 1,
                    ExecutionResult::Mismatch => ui.state.mismatches += 1,
                    ExecutionResult::Timeout => ui.state.timeouts += 1,
                }

                let mut output = None;
                if let ExecutionResult::Crash(out) = &result {
                    output = Some(out.as_str());
                }

                if result.should_save(
                    &options.strategy,
                    options.ignore.iter().chain(&config.fuzzer.ignore),
                ) {
                    save_shader(&options.output, shader, &reconditioned, metadata, output)?;
                    match result {
                        ExecutionResult::Crash(_) => ui.state.saved_crashes += 1,
                        ExecutionResult::Mismatch => ui.state.saved_mismatches += 1,
                        _ => {}
                    }
                }

                ui.render()?;
            }
        }
    };

    std::thread::spawn(move || worker().unwrap());

    loop {
        ui.lock().unwrap().render()?;

        let event = crossterm::event::read()?;
        match event {
            crossterm::event::Event::Key(key) => {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
            crossterm::event::Event::Mouse(_) => {}
            crossterm::event::Event::Resize(_, _) => {}
        }
    }

    {
        disable_raw_mode()?;
        let mut ui = ui.lock().unwrap();
        execute!(ui.terminal.backend_mut(), LeaveAlternateScreen)?;
        ui.terminal.show_cursor()?;
    }

    Ok(())
}

struct Ui<B: Backend> {
    terminal: Terminal<B>,
    state: UiState,
}

#[derive(Default)]
struct UiState {
    total: usize,
    success: usize,
    timeouts: usize,
    crashes: usize,
    saved_crashes: usize,
    mismatches: usize,
    saved_mismatches: usize,
    failures: usize,
}

impl<B: Backend> Ui<B> {
    fn new(terminal: Terminal<B>, state: UiState) -> Self {
        Ui { terminal, state }
    }

    fn render(&mut self) -> eyre::Result<()> {
        fn pc(a: usize, b: usize) -> f32 {
            if b == 0 {
                0.0
            } else {
                a as f32 / b as f32 * 100.0
            }
        }

        self.terminal.draw(|f| {
            let count = self.state.total;
            let success = self.state.success;
            let crashes = self.state.crashes;
            let saved_crashes = self.state.saved_crashes;
            let mismatches = self.state.mismatches;
            let saved_mismatches = self.state.saved_mismatches;
            let timeouts = self.state.timeouts;
            let failures = self.state.failures;

            #[rustfmt::skip]
            let lines = vec![
                Spans::from(format!("total:      {count}")),
                Spans::from(format!("ok:         {success} ({:.2}%)", pc(success, count))),
                Spans::from(format!("crashes:    {crashes} ({:.2}%)", pc(crashes, count))),
                Spans::from(format!("  saved:    {saved_crashes} ({:.2}%)", pc(saved_crashes, crashes))),
                Spans::from(format!("mismatches: {mismatches} ({:.2}%)", pc(mismatches, count))),
                Spans::from(format!("  saved:    {saved_mismatches} ({:.2}%)", pc(saved_mismatches, mismatches))),
                Spans::from(format!("timeouts:   {timeouts} ({:.2}%)", pc(timeouts, count))),
                Spans::from(format!("failures:   {failures} ({:.2}%)", pc(failures, count))),
            ];

            let line_count = lines.len();
            let mut text_width = 0;
            for line in &lines {
                text_width = text_width.max(line.width());
            }

            let block = Block::default()
                .title(" wgslsmith - fuzzer ")
                .borders(Borders::ALL);

            let text = Paragraph::new(lines);

            let frame_area = f.size();
            let text_area = Rect::new(
                frame_area.x + ((frame_area.width - frame_area.x) / 2 - (text_width as u16 / 2)),
                frame_area.y + ((frame_area.height - frame_area.y) / 2 - (line_count as u16 / 2)),
                text_width as u16,
                line_count as u16,
            );

            f.render_widget(block, frame_area);
            f.render_widget(text, text_area);
        })?;
        Ok(())
    }
}
