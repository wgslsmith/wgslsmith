use std::borrow::Cow;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use reflection::{PipelineDescription, ResourceKind};
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Instance, Limits, Maintain, MapMode, ShaderModuleDescriptor, ShaderSource,
};

use crate::ConfigId;

pub fn get_adapters() -> Vec<types::Adapter> {
    Instance::new(Backends::all())
        .enumerate_adapters(Backends::all())
        .filter_map(|adapter| {
            let info = adapter.get_info();
            Some(types::Adapter {
                name: info.name,
                device_id: info.device,
                backend: match info.backend {
                    wgpu::Backend::Empty => return None,
                    wgpu::Backend::Vulkan => crate::BackendType::Vulkan,
                    wgpu::Backend::Metal => crate::BackendType::Metal,
                    wgpu::Backend::Dx12 => crate::BackendType::Dx12,
                    wgpu::Backend::Dx11 => return None,
                    wgpu::Backend::Gl => return None,
                    wgpu::Backend::BrowserWebGpu => return None,
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
) -> Result<Vec<Vec<u8>>> {
    let backend = match config.backend {
        crate::BackendType::Dx12 => wgpu::Backend::Dx12,
        crate::BackendType::Metal => wgpu::Backend::Metal,
        crate::BackendType::Vulkan => wgpu::Backend::Vulkan,
    };

    let instance = Instance::new(Backends::all());
    let adapter = instance
        .enumerate_adapters(Backends::all())
        .find(|adapter| {
            let info = adapter.get_info();
            info.device == config.device_id && info.backend == backend
        })
        .ok_or_else(|| eyre!("no adapter found matching id: {config}"))?;

    let device_descriptor = DeviceDescriptor {
        limits: Limits {
            // This is needed to support swiftshader
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
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Owned(preprocessed)),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        entry_point: "main",
        label: None,
        module: &shader,
        layout: None,
    });

    let mut buffer_sets = vec![];

    enum BufferSet {
      Storage {
        binding: u32,
        size: usize,
        storage: Buffer,
        read: Buffer,
      },
      Uniform {
        binding: u32,
        buffer: Buffer,
      },
    }

    for resource in &meta.resources {
        let size = resource.size as usize;
        match resource.kind {
            ResourceKind::StorageBuffer => {
                let storage = device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                    size: size as u64,
                    mapped_at_creation: true,
                });

                if let Some(init) = resource.init.as_deref() {
                    storage 
                        .slice(..)
                        .get_mapped_range_mut()
                        .copy_from_slice(init);
                }

                storage.unmap();

                let read = device.create_buffer(&BufferDescriptor {
                    label: None,
                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                    size: size as u64,
                    mapped_at_creation: false,
                });

                buffer_sets.push(BufferSet::Storage {
                    binding: resource.binding,
                    size,
                    storage,
                    read,
                });
            }
            ResourceKind::UniformBuffer => {
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

                buffer_sets.push(BufferSet::Uniform {
                    binding: resource.binding,
                    buffer,
                });
            }
        }
    }

    let bind_group_entries = buffer_sets
        .iter()
        .map(|buffer| match buffer {
          BufferSet::Storage { 
            binding, 
            storage, 
            .. 
          } => BindGroupEntry {
            binding: *binding,
            resource: storage.as_entire_binding()
          },
          BufferSet::Uniform { 
            binding, 
            buffer,
            ..
          } => BindGroupEntry { 
            binding: *binding, 
            resource: buffer.as_entire_binding() 
          }
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
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        for buffer in &buffer_sets {
          if let BufferSet::Storage {
            storage,
            read,
            size,
            ..
          } = buffer
          {
            encoder.copy_buffer_to_buffer(
              storage, 
              0, 
              read, 
              0, 
              *size as u64);
          }
        }
        encoder.finish()
    };

    queue.submit(std::iter::once(commands));

    let mut results = vec![];

    for buffer in &buffer_sets {
        if let BufferSet::Storage { read, .. } = buffer {
            let slice = read.slice(..);
            let (tx, rx) = futures::channel::oneshot::channel();

            slice.map_async(MapMode::Read, move |res| {
                tx.send(res).unwrap();
            });

            device.poll(Maintain::Wait);
            rx.await??;

            let bytes = slice.get_mapped_range();

            results.push(bytes.to_vec());
        }
    }

    Ok(results)
}
