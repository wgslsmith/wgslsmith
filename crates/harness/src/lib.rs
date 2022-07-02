mod dawn;
mod server;
mod wgpu;

pub mod cli;
pub mod utils;

use std::process::{Command, Stdio};

use color_eyre::Result;
use frontend::ExecutionEvent;
use futures::executor::block_on;
use reflection::PipelineDescription;
use types::{BackendType, Config, ConfigId, Implementation};

pub trait HarnessHost {
    fn exec_command() -> Command;
}

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

#[derive(bincode::Encode)]
struct ExecutionArgs<'a> {
    pub shader: &'a str,
    pub pipeline_desc: &'a PipelineDescription,
}

#[derive(bincode::Decode)]
struct ExecutionInput {
    pub shader: String,
    pub pipeline_desc: PipelineDescription,
}

#[derive(bincode::Decode, bincode::Encode)]
struct ExecutionOutput {
    pub buffers: Vec<Vec<u8>>,
}

fn execute<Host: HarnessHost, E: FnMut(ExecutionEvent) -> Result<()>>(
    shader: &str,
    pipeline_desc: &PipelineDescription,
    configs: &[ConfigId],
    mut on_event: E,
) -> Result<()> {
    configs.iter().try_for_each(|config| {
        on_event(ExecutionEvent::Start(config.clone()))?;

        let mut child = Host::exec_command()
            .arg(config.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();

        bincode::encode_into_std_write(
            ExecutionArgs {
                shader,
                pipeline_desc,
            },
            &mut stdin,
            bincode::config::standard(),
        )?;

        let output = child.wait_with_output()?;
        if output.status.success() {
            let (output, _): (ExecutionOutput, _) =
                bincode::decode_from_slice(&output.stdout, bincode::config::standard())?;
            on_event(ExecutionEvent::Success(output.buffers))
        } else {
            on_event(ExecutionEvent::Failure(output.stderr))
        }
    })
}

fn execute_config(
    shader: &str,
    pipeline_desc: &PipelineDescription,
    config: &ConfigId,
) -> Result<Vec<Vec<u8>>> {
    match config.implementation {
        Implementation::Dawn => block_on(dawn::run(shader, pipeline_desc, config)),
        Implementation::Wgpu => block_on(wgpu::run(shader, pipeline_desc, config)),
    }
}
