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
    -d, --debug                      Print ast instead of WGSL code
        --enable-fn <ENABLED_FNS>    Enable built-in functions that are disabled by default
    -h, --help                       Print help information
        --log <LOG>                  Logging configuration string (see https://docs.rs/tracing-
                                     subscriber/0.3.7/tracing_subscriber/struct.EnvFilter.html#directives)

```

Logging to stderr can be enabled for debugging purposes by setting `RUST_LOG=info`. See [env_logger](https://docs.rs/env_logger/latest/env_logger/#enabling-logging).
