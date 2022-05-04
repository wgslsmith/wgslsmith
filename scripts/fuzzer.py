#!/usr/bin/env python3

import argparse
import os
import subprocess
import sys

from datetime import datetime
from pathlib import Path


HARNESS_BUILD_TARGET = os.environ.get('HARNESS_BUILD_TARGET')


strategies = {
    "miscompilations": lambda result: result.is_miscompilation(),
    "crashes": lambda result: result.is_crash(),
    "all": lambda result: not result.is_success(),
}


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("-o", "--output", default="out",
                        help="Path to directory in which to save failing test cases")

    parser.add_argument("--strategy", choices=strategies.keys(), default="all",
                        help="Strategy to use when determining which test cases to save")

    return parser.parse_args()


args = parse_args()


def eprint(message: str):
    print(message, file=sys.stderr)


def find_tools():
    project_root = Path(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    bin_dir = project_root.joinpath("target/release")

    if HARNESS_BUILD_TARGET:
        harness_bin_dir = project_root.joinpath(f"harness/target/{HARNESS_BUILD_TARGET}/release")
    else:
        harness_bin_dir = project_root.joinpath("harness/target/release")

    if HARNESS_BUILD_TARGET and "windows" in HARNESS_BUILD_TARGET:
        harness_bin = "harness.exe"
    else:
        harness_bin = "harness"

    tools = {
        "wgslsmith": bin_dir.joinpath("wgslsmith"),
        "reconditioner": bin_dir.joinpath("reconditioner"),
        "harness": harness_bin_dir.joinpath(harness_bin),
    }

    errors = False
    for name, path in tools.items():
        if not path.exists():
            errors = True
            eprint(f"Couldn't find executable for `{name}` at `{path}`")

    if errors:
        return None

    return tools


tools = find_tools()

if not tools:
    eprint("One or more tool executables could not be found")
    exit(1)

print("Detected tools paths:")
for name, path in tools.items():
    print(f"\t{name:14} : {path}")


def gen_shader():
    args = [
        "--block-min-stmts", "1",
        "--block-max-stmts", "1",
        "--max-fns", "3",
    ]

    res = subprocess.run([tools["wgslsmith"], *args], capture_output=True)
    res.check_returncode()
    return res.stdout.decode("utf-8")


def clean_metadata(value: str):
    if value.startswith("//"):
        return value[len("//"):].strip()
    else:
        return value.strip()


def recondition(shader: str):
    res = subprocess.run([tools["reconditioner"]],
                         input=shader.encode("utf-8"), capture_output=True)
    try:
        res.check_returncode()
    except subprocess.CalledProcessError:
        print(res.stderr.decode("utf-8"))
        raise
    return res.stdout.decode("utf-8")


class ExecutionResult:
    def __init__(self, exit_code) -> None:
        self.exit_code = exit_code

    def is_success(self):
        return self.exit_code == 0

    def is_miscompilation(self):
        return self.exit_code == 1

    def is_crash(self):
        return self.exit_code == 101


def exec_shader(shader: str, metadata: str):
    try:
        res = subprocess.run([tools["harness"], "--metadata", metadata],
                             input=shader.encode("utf-8"), timeout=60)
    except subprocess.TimeoutExpired:
        print("timeout expired")
        return True
    return ExecutionResult(res.returncode)


def save_bad_shader(shader: str, reconditioned: str, metadata: str):
    timestamp = datetime.now().strftime("%Y-%m-%d-%H-%M-%S")

    Path(args.output).mkdir(exist_ok=True)

    with open(f"{args.output}/{timestamp}.wgsl", "w") as f:
        f.write(shader)

    with open(f"{args.output}/{timestamp}_reconditioned.wgsl", "w") as f:
        f.write(reconditioned)

    with open(f"{args.output}/{timestamp}.json", "w") as f:
        f.write(metadata)


strategy = args.strategy

print(f"Using strategy: {strategy}")

while True:
    shader = gen_shader()
    metadata, shader = shader.split("\n", maxsplit=1)
    metadata = clean_metadata(metadata)
    reconditioned = recondition(shader)
    result = exec_shader(recondition(shader), metadata)

    if strategies[strategy](result):
        save_bad_shader(shader, reconditioned, metadata)
