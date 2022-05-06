use std::borrow::Cow;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use common::{ResourceKind, ShaderMetadata};
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Limits, Maintain, MapMode, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource,
};

pub async fn run(shader: &str, meta: &ShaderMetadata) -> Result<Vec<Vec<u8>>> {
    let backend = if cfg!(target_os = "windows") {
        Backends::DX12
    } else if cfg!(target_os = "macos") {
        Backends::METAL
    } else if cfg!(target_os = "linux") {
        Backends::VULKAN
    } else {
        Backends::all()
    };

    let instance = Instance::new(backend);

    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or_else(|| eyre!("failed to create adapter"))?;

    let device_descriptor = DeviceDescriptor {
        limits: Limits {
            max_storage_textures_per_shader_stage: 4,
            ..Default::default()
        },
        ..Default::default()
    };

    let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

    let preprocessor_opts = preprocessor::Options {
        concise_stage_attrs: true,
        module_scope_constants: false,
    };

    let preprocessed = preprocessor::preprocess(preprocessor_opts, shader.to_owned());
    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Owned(preprocessed)),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        entry_point: "main",
        label: None,
        module: &shader,
        layout: None,
    });

    let mut buffers = vec![];

    struct IOBuffer {
        binding: u32,
        buffer: Buffer,
        is_storage: bool,
    }

    for resource in &meta.resources {
        match resource.kind {
            ResourceKind::StorageBuffer => {
                let size = resource.size;
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::STORAGE | BufferUsages::MAP_READ,
                    size: size as u64,
                    mapped_at_creation: false,
                });

                buffers.push(IOBuffer {
                    binding: resource.binding,
                    buffer,
                    is_storage: true,
                });
            }
            ResourceKind::UniformBuffer => {
                let size = resource.size;
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::UNIFORM,
                    size: size as u64,
                    mapped_at_creation: true,
                });

                if let Some(init) = resource.init.as_deref() {
                    buffer
                        .slice(..)
                        .get_mapped_range_mut()
                        .copy_from_slice(init);
                }

                buffer.unmap();

                buffers.push(IOBuffer {
                    binding: resource.binding,
                    buffer,
                    is_storage: false,
                });
            }
        }
    }

    let bind_group_entries = buffers
        .iter()
        .map(|buffer| BindGroupEntry {
            binding: buffer.binding,
            resource: buffer.buffer.as_entire_binding(),
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
        encoder.finish()
    };

    queue.submit(std::iter::once(commands));

    let mut results = vec![];
    for buffer in &buffers {
        if buffer.is_storage {
            let slice = buffer.buffer.slice(..);
            let fut = slice.map_async(MapMode::Read);

            device.poll(Maintain::Wait);
            fut.await?;

            results.push(slice.get_mapped_range().to_vec());
        }
    }

    Ok(results)
}
