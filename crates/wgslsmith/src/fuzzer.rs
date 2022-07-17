use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::{Parser, ValueEnum};
use crossbeam_channel::select;
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

use crate::config::Config;
use crate::harness_runner::{self, ExecutionResult, Harness};

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

    /// Disable the fancy terminal dashboard UI.
    #[clap(long, action)]
    disable_tui: bool,

    /// Whether to save random failures (other than execution failures or buffer mismatches).
    ///
    /// This is mostly for debugging.
    #[clap(long, action)]
    save_failures: bool,
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

impl ExecutionResult {
    fn should_save<'a>(
        &self,
        strategy: &SaveStrategy,
        mut ignore: impl Iterator<Item = &'a Regex>,
    ) -> bool {
        match self {
            ExecutionResult::Success => false,
            // ExecutionResult::Timeout => false,
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

    let disable_tui = options.disable_tui;
    let harness = match options
        .server
        .as_deref()
        .or_else(|| config.default_remote())
    {
        Some(server) => Harness::Remote(server.to_owned()),
        None => Harness::Local(
            config
                .harness
                .path
                .clone()
                .map(Ok)
                .unwrap_or_else(std::env::current_exe)?,
        ),
    };

    let (worker_tx, worker_rx) = crossbeam_channel::bounded(1);

    std::thread::spawn(move || {
        worker(config, options, harness, &mut |result| {
            worker_tx.send(result).unwrap()
        })
        .unwrap()
    });

    if disable_tui {
        while let Ok(msg) = worker_rx.recv() {
            match msg {
                WorkerMessage::Log(line) => println!("{line}"),
                WorkerMessage::Result(result) => println!("saved: {}", result.saved),
            }
        }
    } else {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        let ui = Arc::new(Mutex::new(Ui::new(terminal, UiState::default())));

        let (input_tx, input_rx) = crossbeam_channel::bounded(1);

        thread::spawn(move || loop {
            input_tx.send(crossterm::event::read().unwrap()).unwrap();
        });

        let on_result = |result: WorkerResult| {
            let mut ui = ui.lock().unwrap();
            ui.state.total += 1;
            match result.kind {
                WorkerResultKind::Success => ui.state.success += 1,
                WorkerResultKind::Crash => {
                    ui.state.crashes += 1;
                    if result.saved {
                        ui.state.saved_crashes += 1;
                    }
                }
                WorkerResultKind::Mismatch => {
                    ui.state.mismatches += 1;
                    if result.saved {
                        ui.state.saved_mismatches += 1;
                    }
                }
                // WorkerResultKind::Timeout => ui.state.timeouts += 1,
                WorkerResultKind::ReconditionFailure | WorkerResultKind::ExecutionFailure => {
                    ui.state.failures += 1
                }
            }
        };

        loop {
            ui.lock().unwrap().render()?;
            select! {
                recv(input_rx) -> msg => {
                    if let crossterm::event::Event::Key(key) = msg? {
                        if let KeyCode::Char('q') = key.code {
                            break;
                        }
                    }

                    ui.lock().unwrap().render()?;
                }
                recv(worker_rx) -> msg => {
                    match msg? {
                        WorkerMessage::Log(_line) => {},
                        WorkerMessage::Result(result) => on_result(result),
                    }
                }
            }
        }

        {
            disable_raw_mode()?;
            let mut ui = ui.lock().unwrap();
            execute!(ui.terminal.backend_mut(), LeaveAlternateScreen)?;
            ui.terminal.show_cursor()?;
        }
    }

    Ok(())
}

enum WorkerMessage {
    Log(String),
    Result(WorkerResult),
}

struct WorkerResult {
    kind: WorkerResultKind,
    saved: bool,
}

enum WorkerResultKind {
    Success,
    Crash,
    Mismatch,
    // Timeout,
    ReconditionFailure,
    ExecutionFailure,
}

fn worker(
    config: Config,
    options: Options,
    harness: Harness,
    on_message: &mut dyn FnMut(WorkerMessage),
) -> eyre::Result<()> {
    loop {
        let mut logger = |line| on_message(WorkerMessage::Log(line));
        let result = worker_iteration(&config, &options, &harness, &mut logger)?;
        on_message(WorkerMessage::Result(result))
    }
}

fn worker_iteration(
    config: &Config,
    options: &Options,
    harness: &Harness,
    logger: &mut dyn FnMut(String),
) -> eyre::Result<WorkerResult> {
    let shader = gen_shader(options)?;
    let (metadata, shader) = shader
        .split_once('\n')
        .ok_or_else(|| eyre!("expected first line of shader to be a JSON metadata comment"))?;

    let metadata = metadata.trim_start_matches("//").trim();
    let reconditioned = match recondition_shader(shader) {
        Ok(reconditioned) => reconditioned,
        Err(_) => {
            eprintln!("reconditioner command failed, ignoring");
            return Ok(WorkerResult {
                kind: WorkerResultKind::ReconditionFailure,
                saved: false,
            });
        }
    };

    let exec_result = harness_runner::exec_shader(
        harness,
        options.config.as_deref(),
        &reconditioned,
        metadata,
        logger,
    );

    let result = match exec_result {
        Ok(result) => result,
        Err(e) => {
            if options.save_failures {
                save_shader(
                    &options.output,
                    shader,
                    &reconditioned,
                    metadata,
                    Some(&format!("{e:#?}")),
                )?;
            }
            return Ok(WorkerResult {
                kind: WorkerResultKind::ExecutionFailure,
                saved: false,
            });
        }
    };

    let result_kind = match result {
        ExecutionResult::Success => WorkerResultKind::Success,
        ExecutionResult::Crash(_) => WorkerResultKind::Crash,
        ExecutionResult::Mismatch => WorkerResultKind::Mismatch,
        // ExecutionResult::Timeout => WorkerResultKind::Timeout,
    };

    let mut output = None;
    if let ExecutionResult::Crash(out) = &result {
        output = Some(out.as_str());
    }

    let should_save = result.should_save(
        &options.strategy,
        options.ignore.iter().chain(&config.fuzzer.ignore),
    );

    if should_save {
        save_shader(&options.output, shader, &reconditioned, metadata, output)?;
    }

    Ok(WorkerResult {
        kind: result_kind,
        saved: should_save,
    })
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
