use std::borrow::Cow;

use crate::ConfigId;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use reflection::{PipelineDescription, ResourceKind};
use wgpu::wgt::PollType::Wait;
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Limits, MapMode, ShaderModuleDescriptor, ShaderSource,
};

pub fn get_adapters() -> Vec<types::Adapter> {
    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let adapters = futures::executor::block_on(instance.enumerate_adapters(Backends::all()));
    adapters
        .into_iter()
        .filter_map(|adapter| {
            let info = adapter.get_info();
            Some(types::Adapter {
                name: info.name,
                device_id: info.device,
                backend: match info.backend {
                    wgpu::Backend::Vulkan => crate::BackendType::Vulkan,
                    wgpu::Backend::Metal => crate::BackendType::Metal,
                    wgpu::Backend::Dx12 => crate::BackendType::Dx12,
                    wgpu::Backend::Gl => return None,
                    wgpu::Backend::BrowserWebGpu => return None,
                    _ => return None,
                },
            })
        })
        .collect()
}

pub async fn run(
    shader: &str,
    meta: &PipelineDescription,
    config: &ConfigId,
) -> Result<Vec<Vec<u8>>> {
    let backend = match config.backend {
        crate::BackendType::Dx12 => wgpu::Backend::Dx12,
        crate::BackendType::Metal => wgpu::Backend::Metal,
        crate::BackendType::Vulkan => wgpu::Backend::Vulkan,
    };

    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(Backends::all()).await;
    let adapter = adapters
        .into_iter()
        .find(|adapter| {
            let info = adapter.get_info();
            info.device == config.device_id && info.backend == backend
        })
        .ok_or_else(|| eyre!("no adapter found matching id: {config}"))?;

    let device_descriptor = DeviceDescriptor {
        required_limits: Limits {
            // This is needed to support swiftshader
            max_storage_textures_per_shader_stage: 4,
            ..Default::default()
        },
        ..Default::default()
    };

    let (device, queue) = adapter.request_device(&device_descriptor).await?;

    let preprocessor_opts = preprocessor::Options {
        module_scope_constants: false,
    };

    let preprocessed = preprocessor::preprocess(preprocessor_opts, shader.to_owned());
    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Owned(preprocessed)),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        entry_point: Some("main"),
        label: None,
        module: &shader_module,
        layout: None,
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    let mut resource_buffers = vec![];

    enum ResourceBuffer {
        Storage {
            binding: u32,
            size: u64,
            gpu_buffer: Buffer,
            staging_buffer: Buffer,
        },
        Uniform {
            binding: u32,
            buffer: Buffer,
        },
    }

    for resource in &meta.resources {
        let size = resource.size as u64;
        match resource.kind {
            ResourceKind::StorageBuffer => {
                let gpu_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Storage GPU Buffer"),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                    size,
                    mapped_at_creation: false,
                });

                let staging_buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Storage Staging Buffer"),
                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                    size,
                    mapped_at_creation: false,
                });

                resource_buffers.push(ResourceBuffer::Storage {
                    binding: resource.binding,
                    size,
                    gpu_buffer,
                    staging_buffer,
                });
            }
            ResourceKind::UniformBuffer => {
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Uniform Buffer"),
                    usage: BufferUsages::UNIFORM,
                    size,
                    mapped_at_creation: true,
                });

                if let Some(init) = resource.init.as_deref() {
                    buffer
                        .slice(..)
                        .get_mapped_range_mut()
                        .copy_from_slice(init);
                }

                buffer.unmap();

                resource_buffers.push(ResourceBuffer::Uniform {
                    binding: resource.binding,
                    buffer,
                });
            }
        }
    }

    let bind_group_entries = resource_buffers
        .iter()
        .map(|res| match res {
            ResourceBuffer::Storage {
                binding,
                gpu_buffer,
                ..
            } => BindGroupEntry {
                binding: *binding,
                resource: gpu_buffer.as_entire_binding(),
            },
            ResourceBuffer::Uniform {
                binding, buffer, ..
            } => BindGroupEntry {
                binding: *binding,
                resource: buffer.as_entire_binding(),
            },
        })
        .collect::<Vec<_>>();

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &pipeline.get_bind_group_layout(0),
        label: None,
        entries: &bind_group_entries,
    });

    let commands = {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        for res in &resource_buffers {
            if let ResourceBuffer::Storage {
                size,
                gpu_buffer,
                staging_buffer,
                ..
            } = res
            {
                encoder.copy_buffer_to_buffer(gpu_buffer, 0, staging_buffer, 0, *size);
            }
        }

        encoder.finish()
    };

    let submission_index = queue.submit(std::iter::once(commands));

    let mut pending_mappings = vec![];

    for res in &resource_buffers {
        if let ResourceBuffer::Storage { staging_buffer, .. } = res {
            let slice = staging_buffer.slice(..);
            let (tx, rx) = futures::channel::oneshot::channel();

            slice.map_async(MapMode::Read, move |res| {
                // ignore send errors if receiver dropped
                let _ = tx.send(res);
            });

            pending_mappings.push((rx, slice, staging_buffer));
        }
    }

    device.poll(Wait {
        submission_index: Some(submission_index),
        timeout: None,
    })?;

    let mut results = vec![];

    for (rx, slice, raw_buffer) in pending_mappings {
        let map_result = rx.await?;
        map_result?;

        let bytes = slice.get_mapped_range();
        results.push(bytes.to_vec());

        drop(bytes);
        raw_buffer.unmap();
    }

    Ok(results)
}
