# wgslsmith

[![CI](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml)
[![](https://img.shields.io/badge/docs-wgslsmith.github.io-orange)](https://wgslsmith.github.io)

`wgslsmith` is a program generator capable of producing random [WGSL](https://www.w3.org/TR/WGSL/) shaders, primarily for fuzzing WGSL compilers. This repository contains the wgslsmith generator source code, as well as tools for testing WGSL/WebGPU implementations.

Currently, the compilers that are supported for testing include [naga](https://github.com/gfx-rs/naga) via [wgpu](https://github.com/gfx-rs/wgpu) and [tint](https://dawn.googlesource.com/tint) via [dawn](https://dawn.googlesource.com/dawn).

## Requirements

- [Rust](https://rustup.rs/)

A more complete list of requirements is available [here](https://wgslsmith.github.io/building/index.html).

## Building

The full set of tools requires compiling Dawn and wgpu. Full instructions can be found in the [docs](https://wgslsmith.github.io/building/index.html).

Alternatively, some of the tools can be used without any WebGPU implementations/compilers (e.g. generator and reconditioner). To compile these, run:

```sh
$ git clone --recursive https://github.com/wgslsmith/wgslsmith
$ cd wgslsmith
$ cargo build -p wgslsmith --release
$ target/release/wgslsmith --help
```

## Usage

All the tools can be used through the `wgslsmith` command:

```sh
# Do some fuzzing
$ wgslsmith fuzz
# Recondition a shader
$ wgslsmith recondition /path/to/shader.wgsl
# Reduce a crash
$ wgslsmith reduce crash path/to/shader.wgsl --config wgpu:dx12:9348 --regex '...'
# Run a shader
$ wgslsmith harness run path/to/shader.wgsl
```

Some options can be configured through a config file. Run `wgslsmith config` to open the default config file in a text editor. You can also specify a custom config file with the `--config-file` option.

```toml
[reducer]
tmpdir = "/home/hasan/dev/wgslsmith/tmp"
parallelism = 24

[reducer.creduce]
path = "/optional/path/to/creduce"

[reducer.cvise]
path = "/optional/path/to/cvise"

[reducer.perses]
# You need this if you want to reduce with perses
jar = "/path/to/perses_deploy.jar"
```

To use perses for reduction, grab and build it from https://github.com/wgslsmith/perses, then add it to the config as above.

## Development

[Insta](https://github.com/mitsuhiko/insta) is used for snapshot testing the parser.

Install the tool with `cargo install cargo-insta` and use `cargo insta test -p parser` to run the parser tests.
