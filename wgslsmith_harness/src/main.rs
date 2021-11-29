use std::borrow::Cow;
use std::io::Read;
use std::mem::size_of;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Maintain, MapMode, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    pollster::block_on(run())?;
    Ok(())
}

#[repr(C)]
#[derive(Debug)]
struct Buffer<const N: usize>([u32; N]);

async fn run() -> Result<()> {
    let mut shader_source = String::new();
    std::io::stdin().read_to_string(&mut shader_source)?;

    let instance = Instance::new(Backends::DX12);

    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or_else(|| eyre!("failed to create adapter"))?;

    let info = adapter.get_info();

    println!("{:#?}", info);

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await?;

    let output = device.create_buffer(&BufferDescriptor {
        label: None,
        usage: BufferUsages::STORAGE | BufferUsages::MAP_READ,
        size: size_of::<Buffer<1>>() as _,
        mapped_at_creation: false,
    });

    println!("----- BEGIN SHADER -----");
    print!("{}", shader_source);
    println!("----- END SHADER -------");

    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Owned(shader_source)),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        entry_point: "main",
        label: None,
        module: &shader,
        layout: None,
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

    let data = slice
        .get_mapped_range()
        .chunks_exact(4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
        .collect::<Vec<_>>();

    println!("{:?}", data);

    Ok(())
}
