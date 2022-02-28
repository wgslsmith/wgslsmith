use std::env;
use std::fs::File;
use std::path::Path;

use clap::Command;

const WIN_SDK_DIR: &str = "/mnt/c/Program Files (x86)/Windows Kits/10";
const MSVC_TOOLS_DIR: &str =
    "/mnt/c/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC";

fn main() {
    dotenv::dotenv().unwrap();

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

    let cwd = std::env::current_dir().unwrap();
    let dawn_dir = cwd.join("dawn/external/dawn");

    let gclient_cfg_tmpl = dawn_dir.join("scripts/standalone.gclient");
    let gclient_cfg = dawn_dir.join(".gclient");

    if !dawn_dir.exists() || std::fs::read_dir(&dawn_dir).unwrap().next().is_none() {
        let status = std::process::Command::new("git")
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .status()
            .unwrap();

        if !status.success() {
            panic!("failed to initialise submodule");
        }
    }

    if !gclient_cfg.exists() {
        println!("Copying scripts/standalone.gclient -> .gclient");
        std::fs::copy(gclient_cfg_tmpl, gclient_cfg).unwrap();
    }
}

fn build_dawn() {
    let target = build_target();

    let cwd = std::env::current_dir().unwrap();
    let dawn_dir = cwd.join("dawn");
    let build_dir = dawn_dir.join("build").join(&target);

    println!("Syncing dawn dependencies");

    // Sync dependencies for dawn
    let status = std::process::Command::new("gclient")
        .arg("sync")
        .current_dir(dawn_dir.join("external/dawn"))
        .status()
        .unwrap();

    if !status.success() {
        panic!("failed to sync dawn dependencies");
    }

    // Create cmake api query file - this tells cmake to generate the codemodel files
    // which contain the dependency information we need to know which libraries to link
    let cmake_api_query = build_dir.join(".cmake/api/v1/query/codemodel-v2");
    std::fs::create_dir_all(build_dir.join(".cmake/api/v1/query/codemodel-v2")).unwrap();
    File::create(&cmake_api_query.join("codemodel-v2")).unwrap();

    println!("Generating cmake build system");

    let mut cmd = std::process::Command::new("cmake");

    cmd.current_dir(&build_dir)
        .arg("-GNinja")
        .arg("-DCMAKE_BUILD_TYPE=Release");

    // If targeting Windows from WSL, use the LLVM cross compilation toolchain
    if target == "x86_64-pc-windows-msvc" && is_wsl() {
        let sdk_version = find_windows_sdk_version().unwrap();

        let toolchain = dawn_dir.join("cmake/WinMsvc.cmake");
        let msvc_base = cwd.join("build/win/msvc");
        let sdk_base = cwd.join("build/win/sdk");

        cmd.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain.display()))
            .arg("-DHOST_ARCH=x86_64")
            .arg("-DLLVM_NATIVE_TOOLCHAIN=/usr/lib/llvm-14")
            .arg(format!("-DMSVC_BASE={}", msvc_base.display()))
            .arg(format!("-DWINSDK_BASE={}", sdk_base.display()))
            .arg(format!("-DWINSDK_VER={}", sdk_version));
    }

    // Run cmake to generate the build system
    let status = cmd.arg("../..").status().unwrap();

    if !status.success() {
        panic!("failed to generate cmake build system");
    }

    println!("Building dawn");

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

    let target = build_target();
    let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let mut cmd = std::process::Command::new("cargo");

    cmd.arg("build")
        .args(["-p", "harness"])
        .arg("--release")
        .env("CARGO_BUILD_TARGET", &target);

    // We use a different target directory when cross compiling to avoid conflicts with rust-analyzer.
    // Otherwise, rust-analyzer will be constantly re-checking all dependencies and cargo will always
    // do a full recompile. Ideally this should probably be fixed in rust-analyzer to support multiple
    // build targets.
    if target != env!("XTASK_HOST_TARGET") {
        cmd.env("CARGO_TARGET_DIR", workspace_dir.join("cross-target"));
    }

    if target == "x86_64-pc-windows-msvc" && is_wsl() {
        let sdk_version = find_windows_sdk_version().unwrap();
        let llvm_path = Path::new("/usr/lib/llvm-14/bin");

        #[rustfmt::skip]
        let cxx_flags = format!("\
            /imsvc {workspace}/build/win/msvc/include \
            /imsvc {workspace}/build/win/sdk/Include/{sdk_version}/ucrt\
            ",
            workspace = workspace_dir.display(),
        );

        #[rustfmt::skip]
        let rustflags = format!("\
            -Lnative={workspace}/build/win/msvc/lib/x64 \
            -Lnative={workspace}/build/win/sdk/Lib/{sdk}/ucrt/x64 \
            -Lnative={workspace}/build/win/sdk/Lib/{sdk}/um/x64 \
            -C linker={linker}\
            ",
            workspace = workspace_dir.display(),
            sdk = sdk_version,
            linker=llvm_path.join("lld").display(),
        );

        cmd.env("CXXFLAGS", cxx_flags)
            .env("RUSTFLAGS", rustflags)
            .env("CXX", llvm_path.join("clang-cl"))
            .env("AR", llvm_path.join("llvm-lib"));
    }

    let status = cmd.status().unwrap();

    if !status.success() {
        panic!("cargo build failed");
    }
}

fn is_wsl() -> bool {
    static mut IS_WSL: Option<bool> = None;
    unsafe {
        if IS_WSL.is_none() {
            IS_WSL = Some(
                std::fs::read_to_string("/proc/sys/kernel/osrelease")
                    .map(|it| it.trim().ends_with("WSL2"))
                    .unwrap_or(false),
            )
        }

        IS_WSL.unwrap()
    }
}

fn build_target() -> String {
    // If a custom target has been set through the HARNESS_BUILD_TARGET environment variable then use
    // that, otherwise fallback to the host target.
    // The value of the host target is set as a variable by the build script - this is necessary since
    // cargo unfortunately only sets the variable when running the build script.
    env::var("HARNESS_BUILD_TARGET").unwrap_or_else(|_| env!("XTASK_HOST_TARGET").to_owned())
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

    println!("Detected Build Tools Versions");
    println!("-----------------------------");
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
