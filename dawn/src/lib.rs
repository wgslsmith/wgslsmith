#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use cxx::UniquePtr;

pub mod webgpu {
    include!(concat!(env!("OUT_DIR"), "/webgpu.rs"));
}

pub type DawnInstanceHandle = UniquePtr<native::DawnInstance>;

#[cxx::bridge]
pub mod native {
    unsafe extern "C++" {
        include!("dawn/src/lib.h");

        type DawnInstance;
        type WGPUDeviceImpl;

        pub fn create_instance() -> UniquePtr<DawnInstance>;
        pub fn create_device(instance: &DawnInstance) -> *mut WGPUDeviceImpl;
    }
}
