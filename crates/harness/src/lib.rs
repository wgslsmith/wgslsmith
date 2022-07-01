mod dawn;
mod server;
mod wgpu;

pub mod cli;
pub mod utils;

use color_eyre::Result;
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

pub struct Execution {
    pub config: ConfigId,
    pub results: Vec<Vec<u8>>,
}

pub fn execute(
    shader: &str,
    meta: &PipelineDescription,
    configs: &[ConfigId],
) -> Result<Vec<Execution>> {
    let mut results = vec![];

    for config in configs {
        frontend::print_pre_execution(config, meta)?;

        let buffers = futures::executor::block_on(execute_config(shader, meta, config))?;

        frontend::print_post_execution(&buffers, meta)?;

        results.push(Execution {
            config: config.clone(),
            results: buffers,
        });
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
