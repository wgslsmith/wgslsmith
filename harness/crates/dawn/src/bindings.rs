use std::ffi::{c_void, CStr, CString};
use std::mem::zeroed;
use std::os::raw::c_char;
use std::ptr::{null, null_mut};

use crate::dawn;
use crate::webgpu::*;
use futures::channel::oneshot;

pub struct Instance(*mut c_void);

pub struct AdapterProperties {
    pub name: String,
    pub backend: WGPUBackendType,
    pub device_id: u32,
}

impl Instance {
    pub fn new() -> Instance {
        Instance(unsafe { dawn::new_instance() })
    }

    pub fn enumerate_adapters(&self) -> Vec<AdapterProperties> {
        #[allow(non_upper_case_globals)]
        unsafe extern "C" fn cb(info: *const WGPUAdapterProperties, userdata: *mut c_void) {
            (userdata as *mut Vec<AdapterProperties>)
                .as_mut()
                .unwrap()
                .push(AdapterProperties {
                    name: CStr::from_ptr((*info).name).to_str().unwrap().to_owned(),
                    backend: (*info).backendType,
                    device_id: (*info).deviceID,
                });
        }

        let mut adapters = vec![];

        unsafe {
            dawn::enumerate_adapters(self.0, Some(cb), &mut adapters as *mut _ as *mut c_void);
        }

        adapters
    }

    pub fn create_device(self, backend: WGPUBackendType, device_id: u32) -> Option<Device> {
        let handle = unsafe { dawn::create_device(self.0, backend, device_id) };

        if handle.is_null() {
            panic!("failed to create dawn device");
        }

        unsafe {
            wgpuDeviceSetUncapturedErrorCallback(handle, Some(default_error_callback), null_mut());
        }

        let device = Device {
            _instance: self,
            handle,
        };

        Some(device)
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            dawn::delete_instance(self.0);
        }
    }
}

pub struct Device {
    _instance: Instance,
    handle: *mut crate::webgpu::WGPUDeviceImpl,
}

impl Device {
    pub fn create_queue(&self) -> DeviceQueue {
        DeviceQueue {
            handle: unsafe { wgpuDeviceGetQueue(self.handle).assert_not_null() },
        }
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

pub struct DeviceQueue {
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

pub struct ShaderModule {
    handle: WGPUShaderModule,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            wgpuShaderModuleRelease(self.handle);
        }
    }
}

pub struct ComputePipeline {
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

pub struct DeviceBuffer {
    handle: WGPUBuffer,
}

bitflags::bitflags! {
    pub struct DeviceBufferUsage: WGPUBufferUsage {
        const STORAGE = WGPUBufferUsage_WGPUBufferUsage_Storage;
        const UNIFORM = WGPUBufferUsage_WGPUBufferUsage_Uniform;
        const COPY_SRC = WGPUBufferUsage_WGPUBufferUsage_CopySrc;
        const COPY_DST = WGPUBufferUsage_WGPUBufferUsage_CopyDst;
        const MAP_READ = WGPUBufferUsage_WGPUBufferUsage_MapRead;
    }
}

bitflags::bitflags! {
    pub struct DeviceBufferMapMode: WGPUMapMode {
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

pub struct BindGroupLayout {
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

pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub buffer: &'a DeviceBuffer,
    pub size: usize,
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

pub struct BindGroup {
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

pub struct CommandEncoder {
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

pub struct ComputePassEncoder {
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

pub struct CommandBuffer {
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
