mod dawn;
mod wgpu;

use color_eyre::Result;
use common::ShaderMetadata;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Implementation {
    Dawn,
    Wgpu,
}

#[derive(Debug)]
pub struct Configuration {
    pub implementation: Implementation,
    pub adapter: Adapter,
}

impl Configuration {
    pub fn all() -> Vec<Configuration> {
        let mut configurations = vec![];

        configurations.extend(
            wgpu::get_adapters()
                .into_iter()
                .map(|adapter| Configuration {
                    implementation: Implementation::Wgpu,
                    adapter,
                }),
        );

        configurations.extend(
            dawn::get_adapters()
                .into_iter()
                .map(|adapter| Configuration {
                    implementation: Implementation::Dawn,
                    adapter,
                }),
        );

        configurations
    }

    pub fn id(&self) -> String {
        let impl_id = match self.implementation {
            Implementation::Dawn => "dawn",
            Implementation::Wgpu => "wgpu",
        };

        let backend_id = match self.adapter.backend {
            BackendType::Dx12 => "dx12",
            BackendType::Metal => "mtl",
            BackendType::Vulkan => "vk",
        };

        let device = self.adapter.device_id;

        format!("{impl_id}:{backend_id}:{device}")
    }
}

pub struct Execution {
    pub dawn: Vec<Vec<u8>>,
    pub wgpu: Vec<Vec<u8>>,
}

pub fn execute(shader: &str, meta: &ShaderMetadata) -> Result<Execution> {
    let out_1 = futures::executor::block_on(dawn::run(shader, meta))?;
    let out_2 = futures::executor::block_on(wgpu::run(shader, meta))?;

    Ok(Execution {
        dawn: out_1,
        wgpu: out_2,
    })
}
