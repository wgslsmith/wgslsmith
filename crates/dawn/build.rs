use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    let root = dunce::canonicalize("../..")?;

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
    let dawn_gen_dir = dawn_build_dir.join("gen");

    println!("cargo:rustc-link-search=native={}", dawn_lib_dir.display());

    let common_libs = [
        "absl_base",
        "absl_int128",
        "absl_log_severity",
        "absl_raw_logging_internal",
        "absl_spinlock_wait",
        "absl_str_format_internal",
        "absl_strings_internal",
        "absl_strings",
        "absl_throw_delegate",
        "dawn_common",
        "dawn_headers",
        "dawn_native",
        "dawn_platform",
        "dawn_proc",
        "dawncpp_headers",
        "SPIRV-Tools-opt",
        "SPIRV-Tools",
        "tint_diagnostic_utils",
        "tint",
    ];

    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY")?;

    for lib in common_libs {
        let lib_name = if target_family == "windows" {
            format!("{lib}.lib")
        } else if target_family == "unix" {
            format!("lib{lib}.a")
        } else {
            panic!("unsupported target_family '{target_family}'");
        };

        let path = dawn_lib_dir.join(lib_name);

        if target_os == "linux"
            && !Command::new("ar")
                .arg("d")
                .arg(&path)
                .arg("Placeholder.cpp.o")
                .status()?
                .success()
        {
            panic!("ar command failed");
        }

        println!("cargo:rerun-if-changed={}", path.display());
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // Additional platform-specific libraries we need to link
    let libs: &[_] = match target_os.as_str() {
        "windows" => &["dxguid"],
        "macos" => &[
            "framework=Foundation",
            "framework=IOKit",
            "framework=IOSurface",
        ],
        "linux" => &["X11"],
        _ => &[],
    };

    for lib in libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let out = PathBuf::from(env::var("OUT_DIR")?);

    let webgpu_header = dawn_gen_dir.join("include/dawn/webgpu.h");

    // Generate bindings for the webgpu header.
    bindgen::builder()
        .header(webgpu_header.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    let mut cc = cc::Build::new();

    cc.file("src/lib.cpp")
        .cpp(true)
        .include(dawn_src_dir.join("include"))
        .include(dawn_gen_dir.join("include"));

    if build_target == "x86_64-pc-windows-msvc" {
        cc.flag("/std:c++17").flag("/MD");
    } else {
        cc.flag("-std=c++17");
    }

    // Compile and link the c++ wrapper code for dawn initialisation.
    cc.compile("dawn_init");

    println!("cargo:rerun-if-changed=src/lib.cpp");

    Ok(())
}
