use std::ffi::CString;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("tint/src/lib.h");
        unsafe fn validate_shader(source: *const c_char) -> bool;
        unsafe fn compile_shader_to_hlsl(source: *const c_char) -> UniquePtr<CxxString>;
    }
}

pub fn validate_shader(source: &str) -> bool {
    let source = CString::new(source).unwrap();
    unsafe { ffi::validate_shader(source.as_ptr()) }
}

pub fn compile_shader_to_hlsl(source: &str) -> String {
    let source = CString::new(source).unwrap();
    unsafe { ffi::compile_shader_to_hlsl(source.as_ptr()) }.to_string()
}
