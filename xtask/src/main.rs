use std::fs::File;
use std::path::Path;

use clap::Command;

const WIN_SDK_DIR: &str = "/mnt/c/Program Files (x86)/Windows Kits/10";
const MSVC_TOOLS_DIR: &str =
    "/mnt/c/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC";

fn main() {
    let matches = Command::new("xtask")
        .subcommand(Command::new("bootstrap"))
        .subcommand(Command::new("build-dawn"))
        .subcommand(Command::new("build-harness"))
        .subcommand_required(true)
        .get_matches();

    match matches.subcommand_name() {
        Some("bootstrap") => bootstrap(),
        Some("build-dawn") => build_dawn(),
        Some("build-harness") => build_harness(),
        _ => unreachable!(),
    }
}

fn bootstrap() {
    if is_wsl() {
        symlink_windows_sdk();
    }
}

fn build_dawn() {
    let sdk_version = find_windows_sdk_version().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let dawn_dir = cwd.join("dawn");
    let build_dir = dawn_dir.join("build");

    // Create cmake api query file - this tells cmake to generate the codemodel files
    // which contain the dependency information we need to know which libraries to link
    let cmake_api_query = build_dir.join(".cmake/api/v1/query/codemodel-v2");
    std::fs::create_dir_all(build_dir.join(".cmake/api/v1/query/codemodel-v2")).unwrap();
    File::create(&cmake_api_query.join("codemodel-v2")).unwrap();

    let toolchain = dawn_dir.join("cmake/WinMsvc.cmake");
    let msvc_base = cwd.join("build/win/msvc");
    let sdk_base = cwd.join("build/win/sdk");

    // Run cmake to generate the build system
    let status = std::process::Command::new("cmake")
        .arg("-GNinja")
        .arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain.display()))
        .arg("-DHOST_ARCH=x86_64")
        .arg("-DLLVM_NATIVE_TOOLCHAIN=/usr/lib/llvm-14")
        .arg(format!("-DMSVC_BASE={}", msvc_base.display()))
        .arg(format!("-DWINSDK_BASE={}", sdk_base.display()))
        .arg(format!("-DWINSDK_VER={}", sdk_version))
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("..")
        .current_dir(&build_dir)
        .status()
        .unwrap();

    if !status.success() {
        panic!("failed to generate cmake build system");
    }

    // Build the dawn_native and dawn_proc targets
    let status = std::process::Command::new("cmake")
        .arg("--build")
        .arg(".")
        .arg("--target")
        .arg("dawn_native")
        .arg("dawn_proc")
        .current_dir(&build_dir)
        .status()
        .unwrap();

    if !status.success() {
        panic!("failed to run cmake build");
    }
}

fn build_harness() {
    bootstrap();

    let sdk_version = find_windows_sdk_version().unwrap();
    let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let target_dir = workspace_dir.join("cross-target");

    let cxx_flags = format!(
        "/imsvc {workspace}/build/win/msvc/include \
        /imsvc {workspace}/build/win/sdk/Include/{sdk_version}/ucrt",
        workspace = workspace_dir.display(),
    );

    let rustflags = format!(
        "-Lnative={workspace}/build/win/msvc/lib/x64 \
        -Lnative={workspace}/build/win/sdk/Lib/{sdk}/ucrt/x64 \
        -Lnative={workspace}/build/win/sdk/Lib/{sdk}/um/x64",
        workspace = workspace_dir.display(),
        sdk = sdk_version,
    );

    let status = std::process::Command::new("cargo")
        .arg("build")
        .args(["-p", "harness"])
        // We use a different target directory to avoid conflicts with rust-analyzer. Otherwise,
        // rust-analyzer will be constantly re-checking all dependencies and cargo will always do a
        // full recompile. Ideally this should probably be fixed in rust-analyzer to support multiple
        // build targets.
        .env("CARGO_TARGET_DIR", target_dir)
        // TODO: Other build targets
        .env("CARGO_BUILD_TARGET", "x86_64-pc-windows-msvc")
        .env("CXXFLAGS_x86_64_pc_windows_msvc", cxx_flags)
        .env("CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS", rustflags)
        .env(
            "CXX_x86_64_pc_windows_msvc",
            "/usr/lib/llvm-14/bin/clang-cl",
        )
        .env("AR_x86_64_pc_windows_msvc", "/usr/lib/llvm-14/bin/llvm-lib")
        .env(
            "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER",
            "/usr/lib/llvm-14/bin/lld",
        )
        .status()
        .unwrap();

    if !status.success() {
        panic!("cargo build failed");
    }
}

fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|it| it.trim().ends_with("WSL2"))
        .unwrap_or(false)
}

fn symlink_windows_sdk() {
    let build_dir = Path::new("build/win");
    std::fs::create_dir_all(build_dir).unwrap();

    let msvc_version = find_msvc_tools_version().expect("failed to find msvc tools");
    let sdk_version = find_windows_sdk_version().expect("failed to find windows sdk");

    let msvc_dir = build_dir.join("msvc");
    let sdk_dir = build_dir.join("sdk");

    if [&msvc_dir, &sdk_dir].iter().all(|it| it.exists()) {
        return;
    }

    println!("WSL detected, assuming you are targeting Windows");

    println!();
    println!("Build Tools Versions");
    println!("--------------------");
    println!("MSVC Tools:  {msvc_version}");
    println!("Windows SDK: {sdk_version}");
    println!();

    symlink(Path::new(MSVC_TOOLS_DIR).join(msvc_version), msvc_dir);
    symlink(Path::new(WIN_SDK_DIR), sdk_dir);
}

fn find_msvc_tools_version() -> Option<String> {
    find_max_file_in_dir(&MSVC_TOOLS_DIR)
}

fn find_windows_sdk_version() -> Option<String> {
    find_max_file_in_dir(&Path::new(WIN_SDK_DIR).join("Include"))
}

fn find_max_file_in_dir(dir: &dyn AsRef<Path>) -> Option<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|it| Some(it.ok()?.file_name().to_str()?.to_owned()))
        .max()
}

#[cfg(target_family = "unix")]
fn symlink(src: impl AsRef<Path>, link: impl AsRef<Path>) {
    std::os::unix::fs::symlink(src, link).unwrap();
}
