use std::io::{self, Write};

use bincode::{Decode, Encode};
use reflection::{PipelineDescription, ResourceKind};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use types::{Config, ConfigId};

#[derive(Decode, Encode)]
pub enum ExecutionEvent {
    UsingDefaultConfigs(Vec<ConfigId>),
    Start(ConfigId),
    Success(Vec<Vec<u8>>),
    Failure(Vec<u8>),
    Timeout,
}

pub enum ExecutionResult {
    Ok,
    Mismatch,
}

#[derive(Default)]
pub struct Printer;

impl Printer {
    pub fn new() -> Printer {
        Printer
    }
}

impl Printer {
    pub fn print_all_configs(&self, configs: Vec<Config>) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        let id_width = configs
            .iter()
            .map(|it| it.id.to_string().len())
            .max()
            .unwrap_or(0);

        let name_width = configs
            .iter()
            .map(|it| it.adapter_name.len())
            .max()
            .unwrap_or(0);

        stdout.set_color(&dimmed())?;

        writeln!(&mut stdout, "{:<id_width$} | Adapter Name", "ID")?;

        for _ in 0..id_width + 1 {
            write!(&mut stdout, "-")?;
        }

        write!(&mut stdout, "+")?;

        for _ in 0..name_width + 1 {
            write!(&mut stdout, "-")?;
        }

        stdout.reset()?;
        writeln!(&mut stdout)?;

        for config in configs {
            let id = config.id;
            let name = config.adapter_name;

            stdout.set_color(&cyan())?;
            write!(&mut stdout, "{id:<id_width$}")?;

            stdout.set_color(&dimmed())?;
            write!(&mut stdout, " | ")?;

            stdout.reset()?;
            writeln!(&mut stdout, "{name}")?;
        }

        Ok(())
    }

    fn print_default_configs(&self, configs: &[ConfigId]) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        write!(&mut stdout, "no configurations specified, using defaults: ")?;

        for (index, config) in configs.iter().enumerate() {
            stdout.set_color(&cyan())?;
            write!(&mut stdout, "{config}")?;
            stdout.reset()?;

            if index < configs.len() - 1 {
                write!(&mut stdout, ", ")?;
            }
        }

        writeln!(&mut stdout)?;
        writeln!(&mut stdout)?;

        Ok(())
    }

    fn print_pre_execution(
        &self,
        config: &ConfigId,
        pipeline_desc: &PipelineDescription,
    ) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        write!(&mut stdout, "executing ")?;

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        writeln!(&mut stdout, "{config}")?;
        stdout.reset()?;

        writeln!(&mut stdout, "inputs:")?;

        let mut no_inputs = true;
        for resource in pipeline_desc.resources.iter() {
            if let Some(init) = &resource.init {
                let group = resource.group;
                let binding = resource.binding;
                writeln!(&mut stdout, "  {group}:{binding} : {init:?}")?;
                no_inputs = false;
            }
        }

        if no_inputs {
            writeln!(&mut stdout, "  none")?;
        }

        Ok(())
    }

    fn print_post_execution(
        &self,
        buffers: &[Vec<u8>],
        pipeline_desc: &PipelineDescription,
    ) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        writeln!(&mut stdout, "outputs:")?;

        let mut no_outputs = true;
        for (index, resource) in pipeline_desc
            .resources
            .iter()
            .filter(|it| it.kind == ResourceKind::StorageBuffer)
            .enumerate()
        {
            let group = resource.group;
            let binding = resource.binding;
            let buffer = &buffers[index];
            writeln!(&mut stdout, "  {group}:{binding} : {buffer:?}")?;
            no_outputs = false;
        }

        if no_outputs {
            writeln!(&mut stdout, "  none")?;
        }

        writeln!(&mut stdout)?;

        Ok(())
    }

    pub fn print_execution_event(
        &self,
        event: &ExecutionEvent,
        pipeline_desc: &PipelineDescription,
    ) -> io::Result<()> {
        match event {
            ExecutionEvent::UsingDefaultConfigs(configs) => self.print_default_configs(configs),
            ExecutionEvent::Start(config) => self.print_pre_execution(config, pipeline_desc),
            ExecutionEvent::Success(buffers) => self.print_post_execution(buffers, pipeline_desc),
            ExecutionEvent::Failure(stderr) => {
                std::io::stdout().write_all(stderr)?;
                println!();
                Ok(())
            }
            ExecutionEvent::Timeout => {
                let mut stdout = StandardStream::stdout(ColorChoice::Auto);
                stdout.set_color(&yellow())?;
                writeln!(stdout, "timeout")?;
                stdout.reset()?;
                writeln!(stdout)?;
                Ok(())
            }
        }
    }

    pub fn print_execution_result(&self, result: ExecutionResult) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        match result {
            ExecutionResult::Ok => {
                stdout.set_color(&green())?;
                writeln!(stdout, "ok")?;
                stdout.reset()?;
            }
            ExecutionResult::Mismatch => {
                stdout.set_color(&red())?;
                writeln!(stdout, "mismatch")?;
                stdout.reset()?;
            }
        }

        Ok(())
    }
}

fn dimmed() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_dimmed(true);
    spec
}

fn cyan() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Cyan));
    spec
}

fn red() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Red));
    spec
}

fn green() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Green));
    spec
}

fn yellow() -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Yellow));
    spec
}
