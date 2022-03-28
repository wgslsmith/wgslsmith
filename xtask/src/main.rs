use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
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

    fn symlink_windows_sdk(&self) -> Result<()> {
        let xwin_cache = self.sh.current_dir().join(".xwin-cache/splat");
        let crt_dir = xwin_cache.join("crt");
        let sdk_dir = xwin_cache.join("sdk");

        let msvc_version =
            find_msvc_tools_version().ok_or_else(|| anyhow!("failed to find msvc tools"))?;
        let sdk_version =
            find_windows_sdk_version().ok_or_else(|| anyhow!("failed to find windows sdk"))?;

        let msvc_tools_dir = Path::new(MSVC_TOOLS_DIR).join(msvc_version);
        let win_sdk_dir = Path::new(WIN_SDK_DIR);

        symlink(msvc_tools_dir.join("include"), crt_dir.join("include"))?;
        symlink(msvc_tools_dir.join("lib/x64"), crt_dir.join("lib/x86_64"))?;

        symlink(
            win_sdk_dir.join("Include").join(&sdk_version),
            sdk_dir.join("include"),
        )?;

        symlink(
            win_sdk_dir.join("Lib").join(&sdk_version).join("ucrt/x64"),
            sdk_dir.join("lib/ucrt/x86_64"),
        )?;

        symlink(
            win_sdk_dir.join("Lib").join(&sdk_version).join("um/x64"),
            sdk_dir.join("lib/um/x86_64"),
        )?;

        Ok(())
    }

    fn xwin_download(&self) -> Result<()> {
        let xwin_cache = Path::new(".xwin-cache");
        if !xwin_cache.exists() {
            cmd!(self.sh, "xwin --accept-license splat --include-debug-libs").run()?;
        }
        Ok(())
    }

    fn bootstrap(&self) -> anyhow::Result<()> {
        if self.build_target()? == "x86_64-pc-windows-msvc" {
            if is_wsl() {
                self.symlink_windows_sdk()?;
            } else {
                self.xwin_download()?;
            }
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

    fn build_dawn(&self) -> anyhow::Result<()> {
        let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

        let dawn_dir = workspace_dir.join("external/dawn");
        let build_dir = workspace_dir
            .join("build")
            .join(self.build_target()?)
            .join("dawn");

        // Sync dependencies for dawn
        let pushed = self.sh.push_dir(&dawn_dir);
        println!("> syncing dawn dependencies");
        cmd!(self.sh, "gclient sync").run()?;
        drop(pushed);

        self.build_cmake(dawn_dir, build_dir, &["dawn_native", "dawn_proc"])?;

        Ok(())
    }

    fn build_cmake(
        &self,
        src_dir: impl AsRef<Path>,
        build_dir: impl AsRef<Path>,
        targets: &[&str],
    ) -> Result<()> {
        let src_dir = src_dir.as_ref();
        let build_dir = build_dir.as_ref();

        std::fs::create_dir_all(build_dir)?;

        let target = self.build_target()?;
        let pushed = self.sh.push_dir(build_dir);

        let mut cmake_args = vec![];

        if target == "x86_64-pc-windows-msvc" {
            println!("> cross compiling for {target}");

            let toolchain = Path::new("cmake/WinMsvc.cmake").canonicalize().unwrap();
            let xwin_cache = Path::new(".xwin-cache").canonicalize().unwrap();

            let toolchain = toolchain.display();
            let xwin_cache = xwin_cache.display();
            let llvm_toolchain = find_llvm_toolchain().expect("failed to find llvm toolchain");

            cmake_args = vec![
                format!("-DCMAKE_TOOLCHAIN_FILE={toolchain}"),
                format!("-DLLVM_NATIVE_TOOLCHAIN={llvm_toolchain}"),
                format!("-DXWIN_CACHE={xwin_cache}"),
            ];
        }

        cmd!(self.sh, "cmake -GNinja -DCMAKE_BUILD_TYPE=Release -DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={build_dir}/lib {cmake_args...} {src_dir}").run()?;

        if targets.is_empty() {
            cmd!(self.sh, "cmake --build .").run()?;
        } else {
            cmd!(self.sh, "cmake --build . --target {targets...}").run()?;
        }

        drop(pushed);

        Ok(())
    }

    fn build_crate(&self, cmd: &str, name: &str, args: &[String]) -> anyhow::Result<()> {
        let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let target = self.build_target()?;

        // Use provided DAWN_LIB_DIR if set, otherwise fall back to workspace build dir
        let dawn_lib_dir = env::var("DAWN_LIB_DIR")
            .map(|it| it.into())
            .unwrap_or_else(|_| workspace_dir.join("build").join(&target).join("dawn/lib"));

        let mut cmd =
            cmd!(self.sh, "cargo {cmd} -p {name} {args...}").env("DAWN_LIB_DIR", dawn_lib_dir);

        if target == "x86_64-pc-windows-msvc" {
            println!("> cross compiling for {target}");

            let llvm_path =
                PathBuf::from(find_llvm_toolchain().expect("failed to find llvm toolchain"));

            let xwin_cache = workspace_dir.join(".xwin-cache/splat");
            let xwin_cache = xwin_cache.display();

            let cxx_flags = [
                format!("/imsvc {xwin_cache}/crt/include"),
                format!("/imsvc {xwin_cache}/sdk/include/ucrt"),
            ];

            let rustflags = [
                format!("-Lnative={xwin_cache}/crt/lib/x86_64"),
                format!("-Lnative={xwin_cache}/sdk/lib/ucrt/x86_64"),
                format!("-Lnative={xwin_cache}/sdk/lib/um/x86_64"),
                format!("-C linker={}", llvm_path.join("bin/lld").display()),
            ];

            cmd = cmd
                .env("CXXFLAGS", cxx_flags.join(" "))
                .env("RUSTFLAGS", rustflags.join(" "))
                .env("CXX", llvm_path.join("bin/clang-cl"))
                .env("AR", llvm_path.join("bin/llvm-lib"));
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

fn find_llvm_toolchain() -> Option<String> {
    let mut entries = std::fs::read_dir("/usr/lib")
        .into_iter()
        .flatten()
        .filter_map(|it| {
            let entry = it.ok()?;
            if entry.file_name().to_string_lossy().starts_with("llvm-") {
                Some(entry.path().to_str()?.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|it| it.split('-').nth(1).unwrap().parse::<u32>().unwrap());
    entries.into_iter().next_back()
}

#[cfg(target_family = "unix")]
fn symlink(src: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = link.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !link.as_ref().exists() {
        println!(
            "> linking: {} -> {}",
            src.as_ref().display(),
            link.as_ref().display()
        );
        std::os::unix::fs::symlink(src, link)?;
    }
    Ok(())
}

#[cfg(not(target_family = "unix"))]
fn symlink(_src: impl AsRef<Path>, _link: impl AsRef<Path>) -> Result<()> {
    unimplemented!();
}
