mod dawn;
mod server;
mod wgpu;

pub mod cli;
pub mod reflection;
pub mod utils;

use std::fmt::{Display, Write as _};
use std::io::Write as _;
use std::str::FromStr;

use color_eyre::Result;
use reflection::{PipelineDescription, ResourceKind};
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendType {
    Dx12 = 3,
    Metal = 4,
    Vulkan = 5,
}

#[derive(Debug)]
pub struct Adapter {
    pub name: String,
    pub device_id: usize,
    pub backend: BackendType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Implementation {
    Dawn,
    Wgpu,
}

#[derive(Clone, Debug)]
pub struct ConfigId {
    implementation: Implementation,
    backend: BackendType,
    device_id: usize,
}

impl FromStr for ConfigId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<ConfigId, Self::Err> {
        let mut tokens = value.split(':');

        let imp = tokens.next().ok_or("missing implementation segment")?;
        let backend = tokens.next().ok_or("missing backend segment")?;
        let device = tokens.next().ok_or("missing device id segment")?;

        if tokens.next().is_some() {
            return Err("unexpected tokens");
        }

        Ok(ConfigId {
            implementation: match imp {
                "dawn" => Implementation::Dawn,
                "wgpu" => Implementation::Wgpu,
                _ => return Err("invalid implementation"),
            },
            backend: match backend {
                "dx12" => BackendType::Dx12,
                "mtl" => BackendType::Metal,
                "vk" => BackendType::Vulkan,
                _ => return Err("invalid backend"),
            },
            device_id: device.parse().map_err(|_| "invalid device id")?,
        })
    }
}

impl Display for ConfigId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let impl_id = match self.implementation {
            Implementation::Dawn => "dawn",
            Implementation::Wgpu => "wgpu",
        };

        let backend_id = match self.backend {
            BackendType::Dx12 => "dx12",
            BackendType::Metal => "mtl",
            BackendType::Vulkan => "vk",
        };

        let device = self.device_id;

        let id_width =
            impl_id.len() + backend_id.len() + ((self.device_id as f64).log10() as usize) + 3;

        write!(f, "{impl_id}:{backend_id}:{device}")?;

        if let Some(width) = f.width() {
            for _ in 0..width - id_width {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Config {
    pub id: ConfigId,
    pub adapter_name: String,
}

impl Config {
    pub fn all() -> Vec<Config> {
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

    fn new(imp: Implementation, adapter: Adapter) -> Self {
        Config {
            id: ConfigId {
                implementation: imp,
                backend: adapter.backend,
                device_id: adapter.device_id,
            },
            adapter_name: adapter.name,
        }
    }
}

pub fn default_configs() -> Vec<ConfigId> {
    let mut configs = vec![];
    let available = Config::all();

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
