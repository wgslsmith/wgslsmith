#!/usr/bin/env python3

import os
import shlex
import shutil
import subprocess

from pathlib import Path
import sys


def load_config():
    cmd = shlex.split("bash -c 'source config.sh && env'")
    output = subprocess.check_output(cmd).decode()
    for line in output.splitlines():
        tokens = line.split("=", maxsplit=1)
        if len(tokens) > 1:
            name, value = tokens
            os.environ[name] = value


def get_cargo_host_target():
    output = subprocess.check_output(["cargo", "-Vv"]).decode()
    for line in output.splitlines():
        if line.startswith("host:"):
            return line.split(":")[1].strip()


def get_commit(git_dir):
    output = subprocess.check_output(["git", "--git-dir", git_dir, "rev-parse", "HEAD"])
    return output.decode().strip()


def read_gclient_sync_hash():
    path = Path("build/dawn/gclient_sync_hash")
    if path.exists():
        return path.read_text().strip()


def write_gclient_sync_hash(hash):
    path = Path("build/dawn/gclient_sync_hash")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(hash)


def gen_cmake_build(src_dir: Path, build_dir: Path, args=[], env={}):
    build_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "cmake",
        "-GNinja",
        "-DCMAKE_BUILD_TYPE=Release",
        f'-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY={build_dir.absolute().joinpath("lib")}',
        *args,
        src_dir.absolute(),
    ]

    cmd_env = os.environ.copy()
    cmd_env.update(env)

    subprocess.run(cmd, cwd=build_dir, env=cmd_env).check_returncode()


def cmake_build(build_dir: Path, targets=[]):
    subprocess.run(
        ["cmake", "--build", ".", "--target", *targets], cwd=build_dir
    ).check_returncode()


host_target = get_cargo_host_target()
harness_target = os.environ.get("HARNESS_TARGET", host_target)

is_cross = host_target != harness_target


def cargo_build(package, target=None, cwd=None):
    cmd = ["cargo", "build", "-p", package, "--release"]
    if target:
        cmd += ["--target", target]
    elif is_cross:
        cmd += ["--target", host_target]
    subprocess.run(cmd, cwd=cwd).check_returncode()


if __name__ == "__main__":
    if len(sys.argv) > 1:
        target = sys.argv[1]
    else:
        target = "all"

    if target not in {"all", "dawn"}:
        print(f"invalid target: {target}")
        exit(1)

    print(f"> target: {target}")

    dawn_commit = get_commit("external/dawn/.git")
    gclient_sync_hash = read_gclient_sync_hash()

    # Bootstrap gclient config if it doesn't exist
    gclient_config = Path("external/dawn/.gclient")
    gclient_config_tmpl = Path("external/dawn/scripts/standalone.gclient")
    if not gclient_config.exists():
        shutil.copyfile(gclient_config_tmpl, gclient_config)

    # Run gclient sync if the dawn commit has changed
    if gclient_sync_hash != dawn_commit:
        print("> dawn commit has changed, rerunning gclient sync")
        subprocess.run(["gclient", "sync"], cwd="external/dawn").check_returncode()
        write_gclient_sync_hash(dawn_commit)

    dawn_src_dir = Path("external/dawn")

    # Generate dawn build system for host compilation
    dawn_build_dir = Path(f"build/dawn/{host_target}")
    if not dawn_build_dir.exists():
        gen_cmake_build(dawn_src_dir, dawn_build_dir)

    # Build tint for the host
    if target == "all":
        print(f"> building tint (target={host_target})")
        cmake_build(dawn_build_dir, ["tint"])

    # Build wgslsmith
    if target == "all":
        print(f"> building wgslsmith (target={host_target})")
        cargo_build("wgslsmith")

    # If the harness is being cross compiled for windows, then we also want to cross compile dawn
    # separately for windows
    if harness_target != host_target:
        if harness_target != "x86_64-pc-windows-msvc":
            print(
                f"building dawn is not supported for non-native target '{harness_target}' (host={host_target})"
            )
            exit(1)

        dawn_build_dir = Path(f"build/dawn/{harness_target}")
        if not dawn_build_dir.exists():
            cmake_args = [
                f"-DLLVM_NATIVE_TOOLCHAIN={os.environ['LLVM_NATIVE_TOOLCHAIN']}",
                f"-DXWIN_CACHE={os.environ['XWIN_CACHE']}",
                f"-DCMAKE_TOOLCHAIN_FILE={Path('cmake/WinMsvc.cmake').absolute()}",
            ]

            env = {"CXXFLAGS": "-Wno-float-equal"}

            gen_cmake_build(
                dawn_src_dir,
                dawn_build_dir,
                cmake_args,
                env,
            )

    # Build dawn libs
    print(f"> building dawn (target={harness_target})")
    cmake_build(dawn_build_dir, ["dawn_native", "dawn_proc"])

    harness_target = harness_target if harness_target != host_target else None

    # Build test harness
    if target == "all":
        print(f"> building harness (target={harness_target})")
        cargo_build("harness", harness_target)
