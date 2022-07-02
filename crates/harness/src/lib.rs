mod dawn;
mod server;
mod wgpu;

pub mod cli;
pub mod utils;

use color_eyre::Result;
use frontend::ExecutionEvent;
use futures::executor::block_on;
use reflection::PipelineDescription;
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

pub fn execute(
    shader: &str,
    pipeline_desc: &PipelineDescription,
    configs: &[ConfigId],
    mut on_event: impl FnMut(ExecutionEvent) -> Result<()>,
) -> Result<()> {
    configs
        .iter()
        .try_for_each(|config| execute_config(shader, pipeline_desc, config, &mut on_event))
}

pub fn execute_config(
    shader: &str,
    pipeline_desc: &PipelineDescription,
    config: &ConfigId,
    mut on_event: impl FnMut(ExecutionEvent) -> Result<()>,
) -> Result<()> {
    on_event(ExecutionEvent::Start(config.clone()))?;
    let buffers = match config.implementation {
        Implementation::Dawn => block_on(dawn::run(shader, pipeline_desc, config)),
        Implementation::Wgpu => block_on(wgpu::run(shader, pipeline_desc, config)),
    }?;
    on_event(ExecutionEvent::End(buffers))?;
    Ok(())
}
