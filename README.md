# WGSLsmith

[![](https://img.shields.io/badge/rust-1.56%2B-orange.svg)](https://rust-lang.org)

A random program generator for fuzzing WGSL.

## Requirements

- [Rust (>= 1.56.0)](https://rustup.rs/)

## Building

```
> git clone https://github.com/hasali19/WGSLsmith
> cd WGSLsmith
> cargo build -p wgslsmith --release
```

## Usage

```
> target/release/wgslsmith --help
wgslsmith

USAGE:
    wgslsmith.exe [OPTIONS] [SEED]

ARGS:
    <SEED>    Optional u64 to seed the random generator

OPTIONS:
    -d, --debug    Print ast instead of WGSL code
    -h, --help     Print help information

```
