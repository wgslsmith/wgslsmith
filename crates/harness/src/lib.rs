mod dawn;
mod server;
mod wgpu;

pub mod cli;
pub mod reflection;
pub mod utils;

use std::io::Write as _;

use color_eyre::Result;
use reflection::{PipelineDescription, ResourceKind};
use termcolor::{Color, ColorSpec, WriteColor};
use types::{BackendType, Config, ConfigId, Implementation};

pub fn query_configs() -> Vec<Config> {
    let mut configurations = vec![];

    configurations.extend(
        wgpu::get_adapters()
            .into_iter()
            .map(|adapter| Config::new(Implementation::Wgpu, adapter)),
    );

    configurations.extend(
        dawn::get_adapters()
            .into_iter()
            .map(|adapter| Config::new(Implementation::Dawn, adapter)),
    );

    configurations
}

pub fn default_configs() -> Vec<ConfigId> {
    let mut configs = vec![];
    let available = query_configs();

    let targets = [
        (Implementation::Dawn, BackendType::Dx12),
        (Implementation::Dawn, BackendType::Metal),
        (Implementation::Dawn, BackendType::Vulkan),
        (Implementation::Wgpu, BackendType::Dx12),
        (Implementation::Wgpu, BackendType::Metal),
        (Implementation::Wgpu, BackendType::Vulkan),
    ];

    for target in targets {
        if let Some(config) = available
            .iter()
            .find(|it| target == (it.id.implementation, it.id.backend))
        {
            configs.push(config.id.clone());
        }
    }

    configs
}

pub struct Execution {
    pub config: ConfigId,
    pub results: Vec<Vec<u8>>,
}

pub fn execute(
    shader: &str,
    meta: &PipelineDescription,
    configs: &[ConfigId],
) -> Result<Vec<Execution>> {
    let mut stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
    let mut results = vec![];

    for config in configs {
        write!(&mut stdout, "executing ")?;

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        writeln!(&mut stdout, "{config}")?;
        stdout.reset()?;

        writeln!(&mut stdout, "inputs:")?;

        for resource in meta.resources.iter() {
            if let Some(init) = &resource.init {
                let group = resource.group;
                let binding = resource.binding;
                writeln!(&mut stdout, "  {group}:{binding} : {init:?}")?;
            }
        }

        let buffers = futures::executor::block_on(execute_config(shader, meta, config))?;

        writeln!(&mut stdout, "outputs:")?;

        for (index, resource) in meta
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

        results.push(Execution {
            config: config.clone(),
            results: buffers,
        });

        writeln!(&mut stdout)?;
    }

    Ok(results)
}

async fn execute_config(
    shader: &str,
    meta: &PipelineDescription,
    config: &ConfigId,
) -> Result<Vec<Vec<u8>>> {
    match config.implementation {
        Implementation::Dawn => dawn::run(shader, meta, config).await,
        Implementation::Wgpu => wgpu::run(shader, meta, config).await,
    }
}
