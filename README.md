# wgslsmith

[![](https://img.shields.io/badge/rust-1.56%2B-orange.svg)](https://rust-lang.org)

`wgslsmith` is a program generator for producing random [WGSL](https://www.w3.org/TR/WGSL/) shader programs, primarily for fuzzing WGSL compilers. This repository contains the wgslsmith generator source code, as well as tools for testing WGSL/WebGPU implementations.

Currently, the compilers that are supported for testing include [naga](https://github.com/gfx-rs/naga) via [wgpu](https://github.com/gfx-rs/wgpu) and [tint](https://dawn.googlesource.com/tint) via [dawn](https://dawn.googlesource.com/dawn).

## Requirements

- [Rust](https://rustup.rs/)

## Usage

Building and running the program generator can be done as follows:

```sh
$ git clone https://github.com/hasali19/wgslsmith
$ cd wgslsmith
$ cargo build -p wgslsmith --release
$ target/release/wgslsmith --help
```

See the [docs](https://wgslsmith.github.io/) for detailed instructions on building and using the test harness and fuzzing tools.
