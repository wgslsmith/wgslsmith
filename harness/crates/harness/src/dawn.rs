use std::ffi::{c_void, CStr, CString};
use std::mem::zeroed;
use std::os::raw::c_char;
use std::ptr::{null, null_mut};

use color_eyre::Result;
use common::{ResourceKind, ShaderMetadata};
use dawn::webgpu::*;
use futures::channel::oneshot;

struct Instance(*mut c_void);

impl Instance {
    pub fn new() -> Instance {
        Instance(unsafe { dawn::new_instance() })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            dawn::delete_instance(self.0);
        }
    }
}

struct Device {
    _instance: Instance,
    handle: *mut dawn::webgpu::WGPUDeviceImpl,
}

impl Device {
    pub fn create() -> (Device, DeviceQueue) {
        let instance = Instance::new();
        let backend = if cfg!(target_os = "windows") {
            WGPUBackendType_WGPUBackendType_D3D12
        } else if cfg!(target_os = "macos") {
            WGPUBackendType_WGPUBackendType_Metal
        } else if cfg!(target_os = "linux") {
            WGPUBackendType_WGPUBackendType_Vulkan
        } else {
            WGPUBackendType_WGPUBackendType_Null
        };

        let handle = unsafe { dawn::create_device(instance.0, backend) };

        if handle.is_null() {
            panic!("failed to create dawn device");
        }

        unsafe {
            wgpuDeviceSetUncapturedErrorCallback(handle, Some(default_error_callback), null_mut());
        }

        let device = Device {
            _instance: instance,
            handle,
        };

        let queue = DeviceQueue {
            handle: unsafe { wgpuDeviceGetQueue(handle).assert_not_null() },
        };

        (device, queue)
    }

    pub fn create_shader_module(&self, source: &str) -> ShaderModule {
        let source = CString::new(source).unwrap();
        ErrorScope::new(self, "shader module creation failed").execute(|| unsafe {
            let wgsl_descriptor = WGPUShaderModuleWGSLDescriptor {
                chain: WGPUChainedStruct {
                    sType: WGPUSType_WGPUSType_ShaderModuleWGSLDescriptor,
                    ..zeroed()
                },
                source: source.as_ptr() as _,
            };

            let descriptor = WGPUShaderModuleDescriptor {
                nextInChain: &wgsl_descriptor as *const _ as _,
                ..zeroed()
            };

            ShaderModule {
                handle: wgpuDeviceCreateShaderModule(self.handle, &descriptor).assert_not_null(),
            }
        })
    }

    pub fn create_compute_pipeline(
        &self,
        shader_module: &ShaderModule,
        entrypoint: &str,
    ) -> ComputePipeline {
        ErrorScope::new(self, "compute pipeline creation failed").execute(|| unsafe {
            let entrypoint = CString::new(entrypoint).unwrap();
            ComputePipeline {
                handle: wgpuDeviceCreateComputePipeline(
                    self.handle,
                    &WGPUComputePipelineDescriptor {
                        label: null(),
                        nextInChain: null(),
                        layout: null_mut(),
                        compute: WGPUProgrammableStageDescriptor {
                            constantCount: 0,
                            constants: null(),
                            module: shader_module.handle,
                            entryPoint: entrypoint.as_ptr(),
                            nextInChain: null(),
                        },
                    },
                ),
            }
        })
    }

    pub fn create_buffer(
        &self,
        mapped: bool,
        size: usize,
        usage: DeviceBufferUsage,
    ) -> DeviceBuffer {
        ErrorScope::new(self, "buffer creation failed").execute(|| unsafe {
            DeviceBuffer {
                handle: wgpuDeviceCreateBuffer(
                    self.handle,
                    &WGPUBufferDescriptor {
                        label: null(),
                        nextInChain: null(),
                        mappedAtCreation: mapped,
                        size: size as _,
                        usage: usage.bits as _,
                    },
                )
                .assert_not_null(),
            }
        })
    }

    pub fn create_bind_group(
        &self,
        layout: &BindGroupLayout,
        entries: &[BindGroupEntry],
    ) -> BindGroup {
        ErrorScope::new(self, "bind group creation failed").execute(|| unsafe {
            let entries = entries.iter().map(|e| e.into()).collect::<Vec<_>>();
            BindGroup {
                handle: wgpuDeviceCreateBindGroup(
                    self.handle,
                    &WGPUBindGroupDescriptor {
                        nextInChain: null(),
                        label: null(),
                        layout: layout.handle,
                        entries: entries.as_ptr(),
                        entryCount: entries.len() as _,
                    },
                )
                .assert_not_null(),
            }
        })
    }

    pub fn create_command_encoder(&self) -> CommandEncoder {
        ErrorScope::new(self, "command encoder creation failed").execute(|| unsafe {
            CommandEncoder {
                handle: wgpuDeviceCreateCommandEncoder(self.handle, &zeroed()).assert_not_null(),
            }
        })
    }

    pub fn tick(&self) {
        unsafe {
            wgpuDeviceTick(self.handle);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            wgpuDeviceRelease(self.handle);
        }
    }
}

struct DeviceQueue {
    handle: WGPUQueue,
}

impl DeviceQueue {
    pub fn submit(&self, commands: &CommandBuffer) {
        unsafe {
            wgpuQueueSubmit(self.handle, 1, &commands.handle);
        }
    }
}

impl Drop for DeviceQueue {
    fn drop(&mut self) {
        unsafe {
            wgpuQueueRelease(self.handle);
        }
    }
}

struct ShaderModule {
    handle: WGPUShaderModule,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            wgpuShaderModuleRelease(self.handle);
        }
    }
}

struct ComputePipeline {
    handle: WGPUComputePipeline,
}

impl ComputePipeline {
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        unsafe {
            BindGroupLayout {
                handle: wgpuComputePipelineGetBindGroupLayout(self.handle, index).assert_not_null(),
            }
        }
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe {
            wgpuComputePipelineRelease(self.handle);
        }
    }
}

struct DeviceBuffer {
    handle: WGPUBuffer,
}

bitflags::bitflags! {
    struct DeviceBufferUsage: WGPUBufferUsage {
        const STORAGE = WGPUBufferUsage_WGPUBufferUsage_Storage;
        const UNIFORM = WGPUBufferUsage_WGPUBufferUsage_Uniform;
        const COPY_SRC = WGPUBufferUsage_WGPUBufferUsage_CopySrc;
        const COPY_DST = WGPUBufferUsage_WGPUBufferUsage_CopyDst;
        const MAP_READ = WGPUBufferUsage_WGPUBufferUsage_MapRead;
    }
}

bitflags::bitflags! {
    struct DeviceBufferMapMode: WGPUMapMode {
        const READ = WGPUMapMode_WGPUMapMode_Read;
    }
}

impl DeviceBuffer {
    pub fn map_async(&self, mode: DeviceBufferMapMode, size: usize) -> oneshot::Receiver<()> {
        unsafe {
            unsafe extern "C" fn map_callback(
                res: WGPUBufferMapAsyncStatus,
                userdata: *mut c_void,
            ) {
                assert_eq!(
                    res,
                    WGPUBufferMapAsyncStatus_WGPUBufferMapAsyncStatus_Success
                );
                let mut tx = Box::from_raw(userdata as *mut Option<oneshot::Sender<()>>);
                (*tx).take().unwrap().send(()).unwrap();
            }

            let (tx, rx) = oneshot::channel::<()>();
            let tx = Box::new(Some(tx));

            wgpuBufferMapAsync(
                self.handle,
                mode.bits as _,
                0,
                size as _,
                Some(map_callback),
                Box::into_raw(tx) as _,
            );

            rx
        }
    }

    pub fn get_mapped_range(&mut self, size: usize) -> &mut [u8] {
        unsafe {
            let ptr = wgpuBufferGetMappedRange(self.handle, 0, size as _);
            std::slice::from_raw_parts_mut(ptr as _, size)
        }
    }

    pub fn get_const_mapped_range(&self, size: usize) -> &[u8] {
        unsafe {
            let ptr = wgpuBufferGetConstMappedRange(self.handle, 0, size as _);
            std::slice::from_raw_parts(ptr as _, size)
        }
    }

    pub fn unmap(&self) {
        unsafe {
            wgpuBufferUnmap(self.handle);
        }
    }
}

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        unsafe {
            wgpuBufferRelease(self.handle);
        }
    }
}

struct BindGroupLayout {
    handle: WGPUBindGroupLayout,
}

impl BindGroupLayout {}

impl Drop for BindGroupLayout {
    fn drop(&mut self) {
        unsafe {
            wgpuBindGroupLayoutRelease(self.handle);
        }
    }
}

struct BindGroupEntry<'a> {
    binding: u32,
    buffer: &'a DeviceBuffer,
    size: usize,
}

impl<'a> From<&BindGroupEntry<'a>> for WGPUBindGroupEntry {
    fn from(entry: &BindGroupEntry<'a>) -> Self {
        WGPUBindGroupEntry {
            binding: entry.binding,
            buffer: entry.buffer.handle,
            offset: 0,
            size: entry.size as _,
            sampler: null_mut(),
            textureView: null_mut(),
            nextInChain: null(),
        }
    }
}

struct BindGroup {
    handle: WGPUBindGroup,
}

impl BindGroup {}

impl Drop for BindGroup {
    fn drop(&mut self) {
        unsafe {
            wgpuBindGroupRelease(self.handle);
        }
    }
}

struct CommandEncoder {
    handle: WGPUCommandEncoder,
}

impl CommandEncoder {
    pub fn begin_compute_pass(&self) -> ComputePassEncoder {
        unsafe {
            ComputePassEncoder {
                handle: wgpuCommandEncoderBeginComputePass(self.handle, &zeroed())
                    .assert_not_null(),
            }
        }
    }

    pub fn copy_buffer_to_buffer(&self, src: &DeviceBuffer, dst: &DeviceBuffer, size: usize) {
        unsafe {
            wgpuCommandEncoderCopyBufferToBuffer(
                self.handle,
                src.handle,
                0,
                dst.handle,
                0,
                size as _,
            );
        }
    }

    pub fn finish(self) -> CommandBuffer {
        unsafe {
            CommandBuffer {
                handle: wgpuCommandEncoderFinish(self.handle, &zeroed()).assert_not_null(),
            }
        }
    }
}

impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unsafe {
            wgpuCommandEncoderRelease(self.handle);
        }
    }
}

struct ComputePassEncoder {
    handle: WGPUComputePassEncoder,
}

impl ComputePassEncoder {
    pub fn set_pipeline(&self, pipeline: &ComputePipeline) {
        unsafe {
            wgpuComputePassEncoderSetPipeline(self.handle, pipeline.handle);
        }
    }

    pub fn set_bind_group(&self, index: u32, group: &BindGroup) {
        unsafe {
            wgpuComputePassEncoderSetBindGroup(self.handle, index, group.handle, 0, [].as_ptr());
        }
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe {
            wgpuComputePassEncoderDispatchWorkgroups(self.handle, x, y, z);
        }
    }
}

impl Drop for ComputePassEncoder {
    fn drop(&mut self) {
        unsafe {
            wgpuComputePassEncoderEnd(self.handle);
            wgpuComputePassEncoderRelease(self.handle);
        }
    }
}

struct CommandBuffer {
    handle: WGPUCommandBuffer,
}

impl CommandBuffer {}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            wgpuCommandBufferRelease(self.handle);
        }
    }
}

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

pub async fn run(shader: &str, meta: &ShaderMetadata) -> Result<Vec<Vec<u8>>> {
    let (device, queue) = Device::create();
    let shader_module = device.create_shader_module(shader);
    let pipeline = device.create_compute_pipeline(&shader_module, "main");

    let mut buffer_sets = vec![];

    for resource in &meta.resources {
        match resource.kind {
            ResourceKind::StorageBuffer => {
                let size = resource.size;
                let storage = device.create_buffer(
                    false,
                    size,
                    DeviceBufferUsage::STORAGE | DeviceBufferUsage::COPY_SRC,
                );

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
                let size = resource.size;
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
        compute_pass.dispatch(1, 1, 1);
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

            results.push(read.get_const_mapped_range(*size).to_vec());
        }
    }

    Ok(results)
}

trait PointerExt {
    fn assert_not_null(self) -> Self;
}

impl<T> PointerExt for *const T {
    fn assert_not_null(self) -> Self {
        if self.is_null() {
            panic!("null pointer")
        } else {
            self
        }
    }
}

impl<T> PointerExt for *mut T {
    fn assert_not_null(self) -> Self {
        if self.is_null() {
            panic!("null pointer")
        } else {
            self
        }
    }
}

struct ErrorScope<'a> {
    device: &'a Device,
    message: &'a str,
}

impl<'a> ErrorScope<'a> {
    fn new(device: &'a Device, message: &'a str) -> Self {
        ErrorScope { device, message }
    }

    fn execute<T>(mut self, block: impl FnOnce() -> T) -> T {
        unsafe {
            wgpuDevicePushErrorScope(
                self.device.handle,
                WGPUErrorFilter_WGPUErrorFilter_Validation,
            );
        }

        unsafe extern "C" fn callback(
            error_type: WGPUErrorType,
            message: *const c_char,
            userdata: *mut c_void,
        ) {
            let scope = (userdata as *mut ErrorScope).as_mut().unwrap();

            if error_type != WGPUErrorType_WGPUErrorType_Validation {
                return;
            }

            if !message.is_null() {
                let message = CStr::from_ptr(message).to_string_lossy();
                eprintln!("{message}");
            }

            panic!("{}", scope.message);
        }

        let result = block();

        unsafe {
            wgpuDevicePopErrorScope(
                self.device.handle,
                Some(callback),
                &mut self as *mut Self as *mut c_void,
            );
        }

        result
    }
}

unsafe extern "C" fn default_error_callback(
    error_type: WGPUErrorType,
    message: *const c_char,
    _: *mut c_void,
) {
    if !message.is_null() {
        let message = CStr::from_ptr(message).to_string_lossy();
        eprintln!("{message}");
    }

    #[allow(non_upper_case_globals)]
    match error_type {
        WGPUErrorType_WGPUErrorType_Validation => {
            panic!("validation error");
        }
        WGPUErrorType_WGPUErrorType_OutOfMemory => {
            panic!("out of memory");
        }
        WGPUErrorType_WGPUErrorType_DeviceLost => {
            panic!("the dawn device was lost");
        }
        WGPUErrorType_WGPUErrorType_Unknown => {
            panic!("an unknown error occurred");
        }
        _ => {}
    }
}
