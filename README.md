# wgslsmith

[![CI](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml)
[![](https://img.shields.io/badge/docs-wgslsmith.github.io-orange)](https://wgslsmith.github.io)

`wgslsmith` is a program generator for producing random [WGSL](https://www.w3.org/TR/WGSL/) shader programs, primarily for fuzzing WGSL compilers. This repository contains the wgslsmith generator source code, as well as tools for testing WGSL/WebGPU implementations.

Currently, the compilers that are supported for testing include [naga](https://github.com/gfx-rs/naga) via [wgpu](https://github.com/gfx-rs/wgpu) and [tint](https://dawn.googlesource.com/tint) via [dawn](https://dawn.googlesource.com/dawn).

## Requirements

- [Rust](https://rustup.rs/)

## Usage

Building and running the program generator can be done as follows:

```sh
$ git clone --recursive https://github.com/wgslsmith/wgslsmith
$ cd wgslsmith
$ cargo build -p generator --release
$ target/release/generator --help
```

See the [docs](https://wgslsmith.github.io/) for detailed instructions on building and using the test harness and fuzzing tools.

## Development

[Insta](https://github.com/mitsuhiko/insta) is used for snapshot testing the parser.

Install the tool with `cargo install cargo-insta` and use `cargo insta test -p parser` to run the parser tests.

### Git Hooks

To setup git hooks, run:

```
$ git config core.hooksPath .githooks
```

Hooks currently assume that [direnv](https://direnv.net/) is installed.
