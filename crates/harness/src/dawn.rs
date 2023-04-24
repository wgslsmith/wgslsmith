use color_eyre::eyre::eyre;
use dawn::webgpu::{
    WGPUBackendType_WGPUBackendType_D3D12, WGPUBackendType_WGPUBackendType_Metal,
    WGPUBackendType_WGPUBackendType_Vulkan,
};
use dawn::*;
use reflection::{PipelineDescription, ResourceKind};

use crate::ConfigId;

enum BufferSet {
    Storage {
        binding: u32,
        size: usize,
        storage: DeviceBuffer,
        read: DeviceBuffer,
    },
    Uniform {
        binding: u32,
        size: usize,
        buffer: DeviceBuffer,
    },
}

pub fn get_adapters() -> Vec<types::Adapter> {
    Instance::new()
        .enumerate_adapters()
        .into_iter()
        .filter_map(|it| {
            #[allow(non_upper_case_globals)]
            Some(types::Adapter {
                name: it.name,
                device_id: it.device_id as usize,
                backend: match it.backend {
                    WGPUBackendType_WGPUBackendType_D3D12 => crate::BackendType::Dx12,
                    WGPUBackendType_WGPUBackendType_Metal => crate::BackendType::Metal,
                    WGPUBackendType_WGPUBackendType_Vulkan => crate::BackendType::Vulkan,
                    _ => return None,
                },
            })
        })
        .collect()
}

pub async fn run(
    shader: &str,
    workgroups: u32,
    meta: &PipelineDescription,
    config: &ConfigId,
) -> color_eyre::Result<Vec<Vec<u8>>> {
    let backend = match config.backend {
        crate::BackendType::Dx12 => WGPUBackendType_WGPUBackendType_D3D12,
        crate::BackendType::Metal => WGPUBackendType_WGPUBackendType_Metal,
        crate::BackendType::Vulkan => WGPUBackendType_WGPUBackendType_Vulkan,
    };

    let device = Instance::new()
        .create_device(backend, config.device_id as u32)
        .ok_or_else(|| eyre!("no adapter found matching id: {config}"))?;

    let queue = device.create_queue();
    let shader_module = device.create_shader_module(shader);
    let pipeline = device.create_compute_pipeline(&shader_module, "main");

    let mut buffer_sets = vec![];

    for resource in &meta.resources {
        let size = resource.size as usize;
        match resource.kind {
            ResourceKind::StorageBuffer => {
                let mut storage = device.create_buffer(
                    true,
                    size,
                    DeviceBufferUsage::STORAGE | DeviceBufferUsage::COPY_SRC,
                );

                if let Some(init) = resource.init.as_deref() {
                    storage.get_mapped_range(size).copy_from_slice(init);
                }

                storage.unmap();

                let read = device.create_buffer(
                    false,
                    size,
                    DeviceBufferUsage::COPY_DST | DeviceBufferUsage::MAP_READ,
                );

                buffer_sets.push(BufferSet::Storage {
                    binding: resource.binding,
                    size,
                    storage,
                    read,
                });
            }
            ResourceKind::UniformBuffer => {
                let mut buffer = device.create_buffer(true, size, DeviceBufferUsage::UNIFORM);

                if let Some(init) = resource.init.as_deref() {
                    buffer.get_mapped_range(size).copy_from_slice(init);
                }

                buffer.unmap();

                buffer_sets.push(BufferSet::Uniform {
                    binding: resource.binding,
                    size,
                    buffer,
                })
            }
        }
    }

    let bind_group_entries = buffer_sets
        .iter()
        .map(|buffers| match buffers {
            BufferSet::Storage {
                binding,
                size,
                storage,
                ..
            } => BindGroupEntry {
                binding: *binding,
                buffer: storage,
                size: *size,
            },
            BufferSet::Uniform {
                binding,
                size,
                buffer,
            } => BindGroupEntry {
                binding: *binding,
                buffer,
                size: *size,
            },
        })
        .collect::<Vec<_>>();

    let bind_group =
        device.create_bind_group(&pipeline.get_bind_group_layout(0), &bind_group_entries);

    let encoder = device.create_command_encoder();

    {
        let compute_pass = encoder.begin_compute_pass();
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group);
        compute_pass.dispatch(workgroups, 1, 1);
    }

    for buffers in &buffer_sets {
        if let BufferSet::Storage {
            storage,
            read,
            size,
            ..
        } = buffers
        {
            encoder.copy_buffer_to_buffer(storage, read, *size);
        }
    }

    let commands = encoder.finish();

    queue.submit(&commands);

    let mut results = vec![];
    for buffers in &buffer_sets {
        if let BufferSet::Storage { read, size, .. } = buffers {
            let mut rx = read.map_async(DeviceBufferMapMode::READ, *size);

            while rx.try_recv().unwrap().is_none() {
                device.tick();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }

            let bytes = read.get_const_mapped_range(*size);

            results.push(bytes.to_vec());
        }
    }

    Ok(results)
}
