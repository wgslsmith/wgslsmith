set dotenv-load

export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS := "-Lnative=build/win/lib/crt -Lnative=build/win/lib/ucrt -Lnative=build/win/lib/um"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER := "lld"
export CXXFLAGS_x86_64_pc_windows_msvc := "-isystem " + justfile_directory() + "/build/win/include/crt -isystem " + justfile_directory() + "/build/win/include/ucrt"

@default:
    just --list

@bootstrap:
    cd {{justfile_directory()}} && scripts/bootstrap

@build-harness: bootstrap
    cargo build -p harness

@run-harness: build-harness
    target/x86_64-pc-windows-msvc/debug/harness.exe

@clean:
    cargo clean
