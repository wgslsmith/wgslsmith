use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

fn main() {
    let build_target = env::var("TARGET").unwrap();
    let dawn_out = Path::new("../../build")
        .join(&build_target)
        .join("dawn")
        .canonicalize()
        .unwrap();

    let cmake_api_reply_dir = dawn_out.join(".cmake/api/v1/reply");

    println!("cargo:rerun-if-changed={}", cmake_api_reply_dir.display());

    // We use the cmake file api to query cmake to find out the paths to all the static libraries
    // that we need to link against. This requires that an empty file named `codemodel-v2` is
    // contained in the `build/.cmake/api/v1/query` directory, and that cmake has subsequently been
    // run so that it generates the necessary json files.

    // The index contains the name of the codemodel file which was queried.
    let cmake_index_path = find_cmake_index_path(&cmake_api_reply_dir).unwrap();
    let cmake_index_file = File::open(cmake_index_path).unwrap();
    let cmake_index: serde_json::Value = serde_json::from_reader(cmake_index_file).unwrap();

    // The codemodel contains a list of all targets registered with cmake.
    let codemodel_filename = get_codemodel_filename(&cmake_index).unwrap();
    let codemodel_file = File::open(cmake_api_reply_dir.join(codemodel_filename)).unwrap();
    let codemodel: serde_json::Value = serde_json::from_reader(codemodel_file).unwrap();

    // Build a map of target ids to objects.
    let targets = get_targets(&codemodel).unwrap();
    let mut libraries = HashSet::new();

    // Find paths to the json files for our direct dependencies: dawn_native and dawn_proc
    let dawn_native = get_target_path(&targets, "dawn_native").unwrap();
    let dawn_proc = get_target_path(&targets, "dawn_proc").unwrap();

    // Recursively discover the set of all static libraries that we transitively depend on
    collect_static_libraries(&targets, &mut libraries, &cmake_api_reply_dir, dawn_native);
    collect_static_libraries(&targets, &mut libraries, &cmake_api_reply_dir, dawn_proc);

    let libs_dir = dawn_out.join("libs");

    std::fs::create_dir_all(&libs_dir).unwrap();

    println!("cargo:rustc-link-search=native={}", libs_dir.display());

    // Tell rustc to link all dependencies
    for rel_path in libraries {
        // TODO: Make this work for other build targets
        let rel_path = Path::new(&rel_path);
        let name = rel_path.file_name().unwrap();

        let src = dawn_out.join(rel_path);
        let dst = libs_dir.join(name);

        // We need to copy all the libs to a single directly which can be added to the lib search path,
        // since absolute library paths don't seem to work on linux.
        std::fs::copy(dawn_out.join(rel_path), libs_dir.join(name))
            .unwrap_or_else(|_| panic!("failed to copy {} to {}", src.display(), dst.display()));

        // Remove extension from filename
        let lib_name = rel_path.file_stem().unwrap().to_str().unwrap();

        // Remove any platform-specific prefixes
        let lib_name = lib_name.strip_prefix("lib").unwrap_or(lib_name);

        println!("cargo:rustc-link-lib=static={lib_name}");
    }

    // Additional platform-specific libraries we need to link
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let libs: &[&str] = match target_os.as_str() {
        "windows" => &["dxguid"],
        "linux" => &["X11"],
        _ => &[],
    };

    for lib in libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dawn_src = Path::new("../../external/dawn");

    let webgpu_header = dawn_out.join("gen/include/dawn/webgpu.h");

    // Generate bindings for the webgpu header.
    bindgen::builder()
        .header(webgpu_header.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    let mut cc = cc::Build::new();

    cc.file("src/lib.cpp")
        .cpp(true)
        .include(dawn_src.join("include"))
        .include(dawn_out.join("gen/include"));

    if build_target == "x86_64-pc-windows-msvc" {
        cc.flag("/std:c++17").flag("/MD");
    }

    // Compile and link the c++ wrapper code for dawn initialisation.
    cc.compile("dawn_init");

    println!("cargo:rerun-if-changed=src/lib.cpp");
}

fn find_cmake_index_path(reply_dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(reply_dir).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.file_name()?.to_str()?.starts_with("index-") && path.extension()? == "json" {
            Some(path)
        } else {
            None
        }
    })
}

fn get_codemodel_filename(cmake_index: &serde_json::Value) -> Option<&str> {
    cmake_index
        .get("reply")?
        .get("codemodel-v2")?
        .get("jsonFile")?
        .as_str()
}

fn get_targets(codemodel: &serde_json::Value) -> Option<HashMap<&str, &serde_json::Value>> {
    let configuration = codemodel
        .get("configurations")?
        .as_array()?
        .iter()
        .find(|config| config.get("name").and_then(|name| name.as_str()) == Some("Release"))?;

    let targets = configuration
        .get("targets")?
        .as_array()?
        .iter()
        .filter_map(|target| Some((target.get("id")?.as_str()?, target)))
        .collect();

    Some(targets)
}

fn get_target_path<'a>(
    targets: &'a HashMap<&str, &serde_json::Value>,
    name: &str,
) -> Option<&'a str> {
    targets
        .values()
        .find(|target| target.get("name").and_then(|it| it.as_str()) == Some(name))?
        .get("jsonFile")?
        .as_str()
}

fn collect_static_libraries(
    targets: &HashMap<&str, &serde_json::Value>,
    libs: &mut HashSet<String>,
    reply_dir: &Path,
    target_filename: &str,
) {
    let target = File::open(reply_dir.join(target_filename)).unwrap();
    let target: serde_json::Value = serde_json::from_reader(target).unwrap();

    if target.get("type").and_then(|it| it.as_str()) == Some("STATIC_LIBRARY") {
        let artifact_paths = target
            .get("artifacts")
            .and_then(|artifacts| artifacts.as_array())
            .map(|artifacts| artifacts.iter())
            .into_iter()
            .flatten()
            .filter_map(|it| Some(it.get("path")?.as_str()?.to_owned()));
        libs.extend(artifact_paths);
    }

    let dependencies = target
        .get("dependencies")
        .and_then(|it| it.as_array())
        .map(|it| it.iter())
        .into_iter()
        .flatten();

    for dependency in dependencies {
        let id = match dependency.get("id").and_then(|it| it.as_str()) {
            Some(id) => id,
            None => continue,
        };

        let target = match targets.get(id) {
            Some(target) => *target,
            None => continue,
        };

        let filename = match target.get("jsonFile").and_then(|it| it.as_str()) {
            Some(filename) => filename,
            None => continue,
        };

        collect_static_libraries(targets, libs, reply_dir, filename);
    }
}
