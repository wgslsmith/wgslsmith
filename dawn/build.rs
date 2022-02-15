use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=src/lib.cpp");

    let dawn_src = env::var("DAWN_SRC").expect("DAWN_SRC must point to dawn src directory");
    let dawn_out =
        env::var("DAWN_OUT").expect("DAWN_OUT must point to dawn build output directory");

    println!("cargo:rerun-if-changed={}/dawn_native.dll.lib", dawn_out);
    println!("cargo:rerun-if-changed={}/dawn_proc.dll.lib", dawn_out);

    println!("cargo:rustc-link-search={}", dawn_out);
    println!("cargo:rustc-link-lib=dylib=dawn_native.dll");
    println!("cargo:rustc-link-lib=dylib=dawn_proc.dll");

    bindgen::builder()
        .header(format!("{dawn_out}/gen/include/dawn/webgpu.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    cc::Build::new()
        .file("src/lib.cpp")
        .compiler("clang-12")
        .archiver("llvm-lib-12")
        .cpp(true)
        .include(format!("{dawn_src}/include"))
        .include(format!("{dawn_out}/gen/include"))
        .flag("-std=c++17")
        .compile("dawn_init");

    let dawn_out = PathBuf::from(dawn_out);
    let target_dir = get_target_dir();

    copy_dll(&dawn_out, &target_dir, "dawn_native.dll");
    copy_dll(&dawn_out, &target_dir, "dawn_proc.dll");
    copy_dll(&dawn_out, &target_dir, "dawn_platform.dll");
    copy_dll(&dawn_out, &target_dir, "vk_swiftshader.dll");
}

fn get_target_dir() -> PathBuf {
    let mut path = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("target");

    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();
    if host != target {
        path = path.join(target);
    }

    path.join(env::var("PROFILE").unwrap())
}

fn copy_dll(src_dir: &Path, dst_dir: &Path, name: &str) {
    fs::copy(src_dir.join(name), dst_dir.join(name)).unwrap_or_else(|e| {
        panic!(
            "failed to copy dll {name} from {} to {}: {}",
            src_dir.display(),
            dst_dir.display(),
            e
        )
    });
}
