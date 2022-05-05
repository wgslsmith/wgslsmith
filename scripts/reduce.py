#!/usr/bin/env python3

import argparse
import os
import shutil
import subprocess
import sys

from pathlib import Path

THIS_DIR = Path(os.path.dirname(os.path.realpath(__file__)))


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("shader", help="Path to the WGSL shader file to reduce.")
    parser.add_argument(
        "metadata",
        nargs="?",
        help="Path to the JSON metadata file. If not set, the script will look for a JSON file with the same name as the shader.",
    )
    parser.add_argument(
        "--server",
        default="localhost:8080",
        help="Address of harness server. Defaults to localhost:8080.",
    )
    return parser.parse_args()


args = parse_args()


def exit_with(message):
    print(message, file=sys.stderr)
    exit(1)


shader_path = Path(args.shader)

if not shader_path.exists():
    exit_with(f"shader at `{shader_path}` does not exist")

shader_path = shader_path.absolute()

if args.metadata:
    metadata_path = Path(args.metadata)
else:
    metadata_path = Path(shader_path.parent.joinpath(shader_path.stem + ".json"))

if not metadata_path.exists():
    exit_with(f"metadata file at `{metadata_path}` does not exist")

metadata_path = metadata_path.absolute()

tint_path = shutil.which("tint")
naga_path = shutil.which("naga")

if not tint_path:
    exit_with("tint executable not found on path")

if not naga_path:
    exit_with("naga executable not found on path")

env = {}
env.update(os.environ)
env.update(
    WGSLREDUCE_SHADER_NAME=shader_path.name,
    WGSLREDUCE_METADATA_PATH=metadata_path,
    WGSLREDUCE_SERVER=args.server,
    WGSLREDUCE_BIN_PATH=THIS_DIR.parent.joinpath("target/release"),
)

subprocess.Popen(
    ["creduce", THIS_DIR.joinpath("reduce-miscompilation.sh"), shader_path, "--not-c"],
    env=env,
)
