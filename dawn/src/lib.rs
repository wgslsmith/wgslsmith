#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod webgpu {
    include!(concat!(env!("OUT_DIR"), "/webgpu.rs"));
}

#[cxx::bridge]
pub mod native {
    unsafe extern "C++" {
        include!("dawn/src/lib.h");
        type WGPUDeviceImpl;
        pub fn init() -> *mut WGPUDeviceImpl;
    }
}
