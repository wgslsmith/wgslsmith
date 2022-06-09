use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new("../..").canonicalize()?;

    let dawn_src_dir = env::var("DAWN_SRC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("external/dawn"));

    let build_target = env::var("TARGET").unwrap();

    let dawn_build_dir = env::var(format!("DAWN_BUILD_DIR_{}", build_target.replace('-', "_")))
        .map(PathBuf::from)
        .or_else(|_| env::var("DAWN_BUILD_DIR").map(PathBuf::from))
        .unwrap_or_else(|_| root.join("build/dawn").join(&build_target));

    println!("cargo:rerun-if-env-changed=DAWN_SRC_DIR");
    println!("cargo:rerun-if-env-changed=DAWN_BUILD_DIR");

    let dawn_lib_dir = dawn_build_dir.join("lib");

    println!("cargo:rustc-link-search=native={}", dawn_lib_dir.display());

    let libs = ["tint_diagnostic_utils", "tint"];

    let target_family = env::var("CARGO_CFG_TARGET_FAMILY")?;

    for lib in libs {
        let lib_name = if target_family == "windows" {
            format!("{lib}.lib")
        } else if target_family == "unix" {
            format!("lib{lib}.a")
        } else {
            panic!("unsupported target_family '{target_family}'");
        };

        let path = dawn_lib_dir.join(lib_name);

        println!("cargo:rerun-if-changed={}", path.display());
        println!("cargo:rustc-link-lib=static={lib}");
    }

    let mut build = cxx_build::bridge("src/lib.rs");

    build
        .file("src/lib.cpp")
        .include(&dawn_src_dir)
        .include(dawn_src_dir.join("include"))
        .define("TINT_BUILD_WGSL_READER", "1")
        .define("TINT_BUILD_HLSL_WRITER", "1")
        .define("TINT_BUILD_MSL_WRITER", "1");

    if build_target.contains("msvc") {
        build.flag("/std:c++17").flag("/MD");
    } else {
        build.flag("-std=c++17");
    }

    build.compile("tint_ffi");

    Ok(())
}
