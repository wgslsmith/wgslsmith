#!/usr/bin/env python3

import argparse
import os
import shutil
import subprocess

from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("task", nargs="?", default="wgslsmith")
    parser.add_argument("--target")
    parser.add_argument("--install-prefix")
    parser.add_argument("--no-reducer", action="store_true")
    parser.add_argument("--no-harness", action="store_true")
    parser.add_argument("-dawn_path", default="external/dawn")
    return parser.parse_args()


args = parse_args()


def get_cargo_host_target():
    output = subprocess.check_output(["cargo", "-Vv"]).decode()
    for line in output.splitlines():
        if line.startswith("host:"):
            return line.split(":")[1].strip()


root_dir = Path(os.path.realpath(__file__)).parent
host_target = get_cargo_host_target()
build_target = args.target if args.target is not None else host_target
is_cross = args.target is not None and host_target != args.target

dawn_src_dir = Path(args.dawn_path)
dawn_build_dir = Path(f"build/dawn/{build_target}")

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
        "-DTINT_BUILD_HLSL_WRITER=ON",
        "-DTINT_BUILD_MSL_WRITER=ON",
        "-DTINT_BUILD_SPV_WRITER=ON",
        *args,
        src_dir.absolute(),
    ]

    cmd_env = os.environ.copy()
    cmd_env.update(env)

    subprocess.run(cmd, cwd=build_dir, env=cmd_env).check_returncode()


def cmake_build(build_dir: Path, targets=[]):
    cmd = ["cmake", "--build", ".", "--target", *targets]
    print(f">> {' '.join(cmd)}")
    subprocess.run(cmd, cwd=build_dir).check_returncode()


def cargo_build(package, target=None, cwd=None, features=[]):
    cmd = ["./cargo", "build", "-p", package, "--release"]
    if target:
        cmd += ["--target", target]
    if len(features) > 0:
        cmd += ["--features", ",".join(features)]
    print(f">> {' '.join(cmd)}")
    subprocess.run(cmd, cwd=cwd).check_returncode()


def bootstrap_gclient_config():
    gclient_config = Path(f'{dawn_src_dir}/.gclient')
    gclient_config_tmpl = Path(f'{dawn_src_dir}/scripts/standalone.gclient')
    if not gclient_config.exists():
        shutil.copyfile(gclient_config_tmpl, gclient_config)


def gclient_sync():
    dawn_commit = get_commit(f'{dawn_src_dir}/.git')
    print(f'dawn commit is: {dawn_commit}')
    gclient_sync_hash = read_gclient_sync_hash()
    if gclient_sync_hash != dawn_commit:
        print("> dawn commit has changed, rerunning gclient sync")
        subprocess.run(["gclient", "sync"], cwd=dawn_src_dir).check_returncode()
        write_gclient_sync_hash(dawn_commit)

def dawn_gen_cmake():
    if is_cross and build_target != "x86_64-pc-windows-msvc":
        print(f"cannot build dawn for target '{build_target}' (host={host_target})")
        exit(1)

    if not dawn_build_dir.exists():
        if is_cross and build_target == "x86_64-pc-windows-msvc":
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
        else:
            gen_cmake_build(dawn_src_dir, dawn_build_dir)


def build_tint():
    print(f"> building tint (target={build_target})")
    cmake_build(dawn_build_dir, ["tint"])


def build_wgslsmith():
    print(f"> building wgslsmith (target={build_target})")
    features = []
    if not args.no_reducer:
        features.append("reducer")
    if not args.no_harness:
        features.append("harness")
    cargo_build("wgslsmith", target=args.target, features=features)


def build_dawn():
    print(f"> building dawn (target={build_target})")
    cmake_build(dawn_build_dir, ["dawn_native", "dawn_proc"])


def build_harness():
    print(f"> building harness (target={build_target})")
    cargo_build("harness", target=args.target)


if args.task not in {"wgslsmith", "harness", "install"}:
    print(f"invalid task: {args.task}")
    exit(1)

print(f"> task: {args.task}")

if args.task == "install":
    prefix = Path(args.install_prefix if args.install_prefix else "/usr/local/bin")

    wgslsmith = Path("target/release/wgslsmith").absolute()
    link = prefix.joinpath("wgslsmith")

    if not wgslsmith.exists():
        print(f"'{wgslsmith}' does not exist, make sure to run './build.py wgslsmith'")
    elif not link.exists():
        print(f"> linking '{link}' to '{wgslsmith}'")
        link.symlink_to(wgslsmith)
    else:
        print(f"'{link}' already exists")

    exit(0)

tasks = [
    bootstrap_gclient_config,
    gclient_sync,
    dawn_gen_cmake,
]

if args.task == "wgslsmith":
    if not args.no_reducer:
        tasks += [build_tint]
    if not args.no_harness:
        tasks += [build_dawn]
    tasks += [build_wgslsmith]
elif args.task == "harness":
    tasks += [build_dawn, build_harness]

for task in tasks:
    task()

