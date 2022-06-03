use std::os::raw::c_char;

extern "C" {
    pub fn validate_shader(source: *const c_char) -> bool;
}
