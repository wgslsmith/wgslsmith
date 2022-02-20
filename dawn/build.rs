use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build/.cmake/api/v1/reply");

    // We use the cmake file api to query cmake to find out the paths to all the static libraries
    // that we need to link against. This requires that an empty file named `codemodel-v2` is
    // contained in the `build/.cmake/api/v1/query` directory, and that cmake has subsequently been
    // run so that it generates the necessary json files.

    // The index contains the name of the codemodel file which was queried.
    let cmake_index_path = find_cmake_index_path().unwrap();
    let cmake_index_file = File::open(cmake_index_path).unwrap();
    let cmake_index: serde_json::Value = serde_json::from_reader(cmake_index_file).unwrap();

    // The codemodel contains a list of all targets registered with cmake.
    let codemodel_filename = get_codemodel_filename(&cmake_index).unwrap();
    let codemodel_file =
        File::open(Path::new("build/.cmake/api/v1/reply").join(codemodel_filename)).unwrap();
    let codemodel: serde_json::Value = serde_json::from_reader(codemodel_file).unwrap();

    // Build a map of target ids to objects.
    let targets = get_targets(&codemodel).unwrap();
    let mut libraries = HashSet::new();

    // Find paths to the json files for our direct dependencies: dawn_native and dawn_proc
    let dawn_native = get_target_path(&targets, "dawn_native").unwrap();
    let dawn_proc = get_target_path(&targets, "dawn_proc").unwrap();

    // Recursively discover the set of all static libraries that we transitively depend on
    collect_static_libraries(&targets, &mut libraries, dawn_native);
    collect_static_libraries(&targets, &mut libraries, dawn_proc);

    let build_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("build");

    // Tell rustc to link all dependencies
    for name in libraries {
        // TODO: Make this work for other build targets
        let name = name.strip_suffix(".lib").unwrap_or(&name);
        println!("cargo:rustc-link-lib={}/{}", build_dir.display(), name);
    }

    // We also need to explicitly link dxguid.lib since it doesn't appear in any dependencies.
    println!("cargo:rustc-link-lib=dxguid");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dawn_src = "external/dawn";
    let dawn_out = "build/external/dawn";

    // Generate bindings for the webgpu header.
    bindgen::builder()
        .header(format!("{dawn_out}/gen/include/dawn/webgpu.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    let build_target = env::var("TARGET").unwrap();

    // Compile and link the c++ wrapper code for dawn instance initialisation.
    if build_target == "x86_64-pc-windows-msvc" {
        cc::Build::new()
            .file("src/lib.cpp")
            .cpp(true)
            .include(format!("{dawn_src}/include"))
            .include(format!("{dawn_out}/gen/include"))
            .flag("/std:c++17")
            .flag("/MD")
            .compile("dawn_init");
    } else {
        // TODO
    }

    println!("cargo:rerun-if-changed=src/lib.cpp");
}

fn find_cmake_index_path() -> Option<PathBuf> {
    std::fs::read_dir("build/.cmake/api/v1/reply")
        .ok()?
        .find_map(|entry| {
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
    target_filename: &str,
) {
    let target = File::open(format!("build/.cmake/api/v1/reply/{}", target_filename)).unwrap();
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

        collect_static_libraries(targets, libs, filename);
    }
}
