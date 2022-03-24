use std::env;
use std::path::Path;

use anyhow::Result;
use clap::{Arg, Command};
use xshell::{cmd, Shell};

const WIN_SDK_DIR: &str = "/mnt/c/Program Files (x86)/Windows Kits/10";
const MSVC_TOOLS_DIR: &str =
    "/mnt/c/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let matches = Command::new("xtask")
        .subcommand(Command::new("bootstrap"))
        .subcommand(
            Command::new("build")
                .arg(Arg::new("target").required(true))
                .arg(Arg::new("args").multiple_values(true).raw(true)),
        )
        .subcommand(
            Command::new("run")
                .arg(Arg::new("target").required(true))
                .arg(Arg::new("args").multiple_values(true).raw(true)),
        )
        .subcommand_required(true)
        .get_matches();

    let xtask = XTask::new()?;

    match matches.subcommand().unwrap() {
        ("bootstrap", _) => xtask.bootstrap(),
        (cmd @ ("build" | "run"), args) => match args.value_of("target").unwrap() {
            "dawn" => xtask.build_dawn(),
            pkg => xtask.build_crate(cmd, pkg, args.values_of_t("args").as_deref().unwrap_or(&[])),
        },
        _ => unreachable!(),
    }
}

struct XTask {
    sh: Shell,
}

impl XTask {
    fn new() -> anyhow::Result<Self> {
        Ok(XTask { sh: Shell::new()? })
    }

    fn is_dir_empty(&self, path: impl AsRef<Path>) -> Result<bool> {
        self.sh
            .read_dir(path)
            .map(|it| it.is_empty())
            .map_err(anyhow::Error::from)
    }

    fn bootstrap(&self) -> anyhow::Result<()> {
        if is_wsl() {
            symlink_windows_sdk();
        }

        let cwd = self.sh.current_dir();
        let dawn_dir = cwd.join("external/dawn");

        let gclient_cfg_tmpl = dawn_dir.join("scripts/standalone.gclient");
        let gclient_cfg = dawn_dir.join(".gclient");

        if !dawn_dir.exists() || self.is_dir_empty(&dawn_dir)? {
            println!("> checking out submodules");
            cmd!(self.sh, "git submodule update --init").run()?;
        }

        if !gclient_cfg.exists() {
            println!("> creating dawn gclient config");
            self.sh.copy_file(gclient_cfg_tmpl, gclient_cfg)?;
        }

        Ok(())
    }

    fn host_triple(&self) -> anyhow::Result<String> {
        Ok(cmd!(self.sh, "rustc --version --verbose")
            .read()?
            .lines()
            .find(|it| it.starts_with("host: "))
            .unwrap()
            .strip_prefix("host: ")
            .unwrap()
            .to_owned())
    }

    fn build_target(&self) -> anyhow::Result<String> {
        match self.sh.var("CARGO_BUILD_TARGET") {
            Ok(target) => Ok(target),
            Err(_) => self.host_triple(),
        }
    }

    fn build_dawn(&self) -> anyhow::Result<()> {
        let target = self.build_target()?;

        let cwd = self.sh.current_dir();
        let dawn_dir = cwd.join("external/dawn").canonicalize().unwrap();
        let build_dir = cwd.join("build").join(&target).join("dawn");

        // Sync dependencies for dawn
        let pushed = self.sh.push_dir(&dawn_dir);
        println!("> syncing dawn dependencies");
        cmd!(self.sh, "gclient sync").run()?;
        drop(pushed);

        let pushed = self.sh.push_dir(&build_dir);

        // Create cmake api query file - this tells cmake to generate the codemodel files
        // which contain the dependency information we need to know which libraries to link
        self.sh
            .write_file(".cmake/api/v1/query/codemodel-v2/codemodel-v2", b"")?;

        let mut cmake_args = vec![];

        // If targeting Windows from WSL, use the LLVM cross compilation toolchain
        if target == "x86_64-pc-windows-msvc" && is_wsl() {
            println!("> cross compiling for {target}");

            let sdk_version = find_windows_sdk_version().unwrap();

            let toolchain = cwd.join("cmake/WinMsvc.cmake");
            let msvc_base = cwd.join("build/win/msvc");
            let sdk_base = cwd.join("build/win/sdk");

            let toolchain = toolchain.display();
            let msvc_base = msvc_base.display();
            let sdk_base = sdk_base.display();
            let host_arch = "x86_64";
            let llvm_toolchain = "/usr/lib/llvm-14";

            cmake_args = vec![
                format!("-DCMAKE_TOOLCHAIN_FILE={toolchain}"),
                format!("-DHOST_ARCH={host_arch}"),
                format!("-DLLVM_NATIVE_TOOLCHAIN={llvm_toolchain}"),
                format!("-DMSVC_BASE={msvc_base}"),
                format!("-DWINSDK_BASE={sdk_base}"),
                format!("-DWINSDK_VER={sdk_version}"),
            ];
        }

        println!("> generating cmake build system");
        #[rustfmt::skip]
        cmd!(self.sh, "cmake -GNinja -DCMAKE_BUILD_TYPE=Release {cmake_args...} {dawn_dir}").run()?;

        println!("> building dawn");
        cmd!(self.sh, "cmake --build . --target dawn_native dawn_proc").run()?;

        drop(pushed);

        Ok(())
    }

    fn build_crate(&self, cmd: &str, name: &str, args: &[String]) -> anyhow::Result<()> {
        let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let target = self.build_target()?;

        let mut cmd = cmd!(self.sh, "cargo {cmd} -p {name} {args...}");

        if target == "x86_64-pc-windows-msvc" && is_wsl() {
            println!("> cross compiling for {target}");

            let sdk_version = find_windows_sdk_version().unwrap();
            let llvm_path = Path::new("/usr/lib/llvm-14/bin");

            let ws = workspace_dir.display();
            let cxx_flags = [
                format!("/imsvc {ws}/build/win/msvc/include"),
                format!("/imsvc {ws}/build/win/sdk/Include/{sdk_version}/ucrt"),
            ];

            let rustflags = [
                format!("-Lnative={ws}/build/win/msvc/lib/x64"),
                format!("-Lnative={ws}/build/win/sdk/Lib/{sdk_version}/ucrt/x64"),
                format!("-Lnative={ws}/build/win/sdk/Lib/{sdk_version}/um/x64"),
                format!("-C linker={}", llvm_path.join("lld").display()),
            ];

            cmd = cmd
                .env("CXXFLAGS", cxx_flags.join(" "))
                .env("RUSTFLAGS", rustflags.join(" "))
                .env("CXX", llvm_path.join("clang-cl"))
                .env("AR", llvm_path.join("llvm-lib"));
        }

        cmd.run()?;

        Ok(())
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

#[cfg(not(target_family = "unix"))]
fn symlink(_src: impl AsRef<Path>, _link: impl AsRef<Path>) {
    unimplemented!();
}
