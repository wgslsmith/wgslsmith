use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=src/lib.h");
    println!("cargo:rerun-if-changed=src/lib.cpp");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dawn_out = format!("{}/../external/dawn/out/Shared", manifest_dir);

    println!("cargo:rustc-link-search={}", dawn_out);
    println!("cargo:rustc-link-lib=dylib=dawn_native.dll");
    println!("cargo:rustc-link-lib=dylib=dawn_proc.dll");

    bindgen::builder()
        .header("../external/dawn/out/Shared/gen/src/include/dawn/webgpu.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    cxx_build::bridge("src/lib.rs")
        .file("src/lib.cpp")
        .include("../external/dawn/src")
        .include("../external/dawn/src/include")
        .include("../external/dawn/out/Shared/gen/src/include")
        .compile("dawn_wrapper");

    let dawn_out = PathBuf::from(dawn_out);
    let target_dir = get_target_dir();

    copy_dll(&dawn_out, &target_dir, "dawn_native.dll");
    copy_dll(&dawn_out, &target_dir, "dawn_proc.dll");
    copy_dll(&dawn_out, &target_dir, "dawn_platform.dll");
    copy_dll(&dawn_out, &target_dir, "libc++.dll");
}

fn get_target_dir() -> PathBuf {
    PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("target")
        .join(env::var("PROFILE").unwrap())
}

fn copy_dll(src_dir: &Path, dst_dir: &Path, name: &str) {
    fs::copy(src_dir.join(name), dst_dir.join(name)).expect("failed to copy dll");
}
