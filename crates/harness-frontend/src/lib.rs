use std::io::Write;

use bincode::{Decode, Encode};
use reflection_types::{PipelineDescription, ResourceKind};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use types::{Config, ConfigId};

pub fn print_all_configs(configs: Vec<Config>) -> eyre::Result<()> {
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

pub fn print_default_configs(configs: &[ConfigId]) -> eyre::Result<()> {
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

fn print_pre_execution(config: &ConfigId, pipeline_desc: &PipelineDescription) -> eyre::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    write!(&mut stdout, "executing ")?;

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
    writeln!(&mut stdout, "{config}")?;
    stdout.reset()?;

    writeln!(&mut stdout, "inputs:")?;

    for resource in pipeline_desc.resources.iter() {
        if let Some(init) = &resource.init {
            let group = resource.group;
            let binding = resource.binding;
            writeln!(&mut stdout, "  {group}:{binding} : {init:?}")?;
        }
    }

    Ok(())
}

fn print_post_execution(
    buffers: &[Vec<u8>],
    pipeline_desc: &PipelineDescription,
) -> eyre::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    writeln!(&mut stdout, "outputs:")?;

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
    }

    writeln!(&mut stdout)?;

    Ok(())
}

#[derive(Decode, Encode)]
pub enum ExecutionEvent {
    Start(ConfigId),
    End(Vec<Vec<u8>>),
}

pub fn print_execution_event(
    event: &ExecutionEvent,
    pipeline_desc: &PipelineDescription,
) -> eyre::Result<()> {
    match event {
        ExecutionEvent::Start(config) => print_pre_execution(config, pipeline_desc),
        ExecutionEvent::End(buffers) => print_post_execution(buffers, pipeline_desc),
    }
}

pub enum ExecutionResult {
    Ok,
    Mismatch,
}

pub fn print_execution_result(result: ExecutionResult) -> eyre::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    match result {
        ExecutionResult::Ok => {
            stdout.set_color(&green())?;
            writeln!(&mut stdout, "ok")?;
            stdout.reset()?;
        }
        ExecutionResult::Mismatch => {
            stdout.set_color(&red())?;
            writeln!(&mut stdout, "mismatch")?;
            stdout.reset()?;
        }
    }

    Ok(())
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
