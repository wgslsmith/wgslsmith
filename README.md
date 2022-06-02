# wgslsmith

[![CI](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/wgslsmith/wgslsmith/actions/workflows/ci.yml)
[![](https://img.shields.io/badge/docs-wgslsmith.github.io-orange)](https://wgslsmith.github.io)

`wgslsmith` is a program generator for producing random [WGSL](https://www.w3.org/TR/WGSL/) shader programs, primarily for fuzzing WGSL compilers. This repository contains the wgslsmith generator source code, as well as tools for testing WGSL/WebGPU implementations.

Currently, the compilers that are supported for testing include [naga](https://github.com/gfx-rs/naga) via [wgpu](https://github.com/gfx-rs/wgpu) and [tint](https://dawn.googlesource.com/tint) via [dawn](https://dawn.googlesource.com/dawn).

## Requirements

- [Rust](https://rustup.rs/)

## Installation

This repo is divided into two workspaces - the top-level workspace contains the main tools for generating and reconditioning shaders, as well as driving fuzzing and test case reduction. These can be compiled by running the following:

```sh
$ git clone --recursive https://github.com/wgslsmith/wgslsmith
$ cd wgslsmith
$ cargo build --release
```

Then add the `bin` directory to your `PATH`. This will make the `wgslsmith` command available, which is the entrypoint to all the tools:

```sh
$ wgslsmith --help
```

In addition to this, the `harness` subdirectory contains the testing harness. This is organised as a separate workspace to improve cross-compilation workflows. Compiling this is a bit more complicated as it requires compiling dawn. See the [docs](https://wgslsmith.github.io/) for detailed instructions.

## Development

[Insta](https://github.com/mitsuhiko/insta) is used for snapshot testing the parser.

Install the tool with `cargo install cargo-insta` and use `cargo insta test -p parser` to run the parser tests.

### Git Hooks

To setup git hooks, run:

```
$ git config core.hooksPath .githooks
```

Hooks currently assume that [direnv](https://direnv.net/) is installed.
