use std::borrow::Cow;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use common::{DataTypeExt, ResourceKind, ShaderMetadata};
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Maintain, MapMode, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};

pub async fn run(shader: &str, meta: &ShaderMetadata) -> Result<Vec<Vec<u8>>> {
    let instance = Instance::new(Backends::VULKAN);

    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or_else(|| eyre!("failed to create adapter"))?;

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await?;

    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(shader)),
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
                let size = resource.description.size();
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
                let size = resource.description.size();
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::UNIFORM,
                    size: size as u64,
                    mapped_at_creation: true,
                });

                // TODO: Set random input
                // buffer.slice(..).get_mapped_range_mut()[0] = 123;

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
            pass.dispatch(1, 1, 1);
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
