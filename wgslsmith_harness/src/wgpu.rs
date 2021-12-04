use std::borrow::Cow;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Maintain, MapMode, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};

use crate::Buffer;

pub async fn run(shader: &str) -> Result<Buffer<1>> {
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

    let output = device.create_buffer(&BufferDescriptor {
        label: None,
        usage: BufferUsages::STORAGE | BufferUsages::MAP_READ,
        size: Buffer::<1>::SIZE as _,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &pipeline.get_bind_group_layout(0),
        label: None,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: output.as_entire_binding(),
        }],
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

    let slice = output.slice(..);
    let fut = slice.map_async(MapMode::Read);

    device.poll(Maintain::Wait);
    fut.await?;

    let data = Buffer::from_bytes(&*slice.get_mapped_range());

    Ok(data)
}
