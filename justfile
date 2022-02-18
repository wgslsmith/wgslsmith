set dotenv-load

export CARGO_BUILD_TARGET := "x86_64-pc-windows-msvc"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS := "-Lnative=build/win/msvc/lib/x64 -Lnative=build/win/sdk/Lib/" + `cat build/win/sdk.version` + "/ucrt/x64 -Lnative=build/win/sdk/Lib/" + `cat build/win/sdk.version` + "/um/x64"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER := "/usr/lib/llvm-14/bin/lld"
export CXX_x86_64_pc_windows_msvc := "/usr/lib/llvm-14/bin/clang-cl"
export CXXFLAGS_x86_64_pc_windows_msvc := "/imsvc " + justfile_directory() + "/build/win/msvc/include /imsvc " + justfile_directory() + "/build/win/sdk/Include/" + `cat build/win/sdk.version` + "/ucrt"
export AR_x86_64_pc_windows_msvc := "/usr/lib/llvm-14/bin/llvm-lib"

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
