use std::fmt::{Display, Write};
use std::str::FromStr;

use bincode::{Decode, Encode};

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Eq)]
pub enum Implementation {
    Dawn,
    Wgpu,
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Eq)]
pub enum BackendType {
    Dx12 = 3,
    Metal = 4,
    Vulkan = 5,
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct ConfigId {
    pub implementation: Implementation,
    pub backend: BackendType,
    pub device_id: usize,
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
pub struct Adapter {
    pub name: String,
    pub device_id: usize,
    pub backend: BackendType,
}

#[derive(Debug, Decode, Encode)]
pub struct Config {
    pub id: ConfigId,
    pub adapter_name: String,
}

impl Config {
    pub fn new(imp: Implementation, adapter: Adapter) -> Self {
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
