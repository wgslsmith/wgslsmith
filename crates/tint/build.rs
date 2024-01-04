use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;

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

    let libs = fs::read_dir(dawn_lib_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|e| e.is_file())
        .filter(|e| e.to_str().unwrap().contains("tint"))
        .collect::<Vec<_>>();
    
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY")?;

    for lib in libs {
        
        let lib_name = lib.file_stem().unwrap().to_str().unwrap();
        
        let lib_name = if target_family == "windows" {
            lib_name
        } else if target_family == "unix" {
            &lib_name[3..]
        } else {
            panic!("unsupported target_family '{target_family}'");
        };

        println!("cargo:rerun-if-changed={}", lib.display());
        println!("cargo:rustc-link-lib=static={}", lib_name);
        
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
