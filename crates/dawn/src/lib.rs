#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings;

pub use bindings::*;

pub mod webgpu {
    include!(concat!(env!("OUT_DIR"), "/webgpu.rs"));
}

mod dawn {
    use std::ffi::c_void;

    use crate::webgpu;

    pub type EnumerateAdapterCallback =
        unsafe extern "C" fn(*const webgpu::WGPUAdapterInfo, *mut c_void);

    extern "C" {
        pub fn new_instance() -> *mut c_void;

        pub fn delete_instance(instance: *mut c_void);

        pub fn instance_process_events(instance: *const c_void);

        pub fn enumerate_adapters(
            instance: *mut c_void,
            callback: Option<EnumerateAdapterCallback>,
            userdata: *mut c_void,
        );

        pub fn create_device(
            instance: *mut c_void,
            backend_type: webgpu::WGPUBackendType,
            device_id: u32,
        ) -> webgpu::WGPUDevice;
    }
}
