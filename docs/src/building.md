# Building wgslsmith

```admonish info
Building for Windows is supported by cross compiling from Linux (ideally using [WSL](https://docs.microsoft.com/en-us/windows/wsl/)). It shouldn't be too hard to build everything natively on Windows, but you're on your own.
```

## Dawn

The test harness depends on [dawn](https://dawn.googlesource.com/dawn), which has a few prerequisites for building:

### CMake

If you don't already have it, run the following (for Ubuntu):

```sh
$ sudo apt install cmake
```

Otherwise, download it from [https://cmake.org/download/](https://cmake.org/download/).

### depot_tools

Dawn uses depot_tools to fetch its dependencies. Detailed usage instructions can be found [here](https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up). To install it:

```sh
# Clone the depot_tools repo somewhere on your system
$ git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git
# Add it to your PATH
$ export PATH=/path/to/depot_tools:$PATH
```

### Ninja

On Ubuntu:

```sh
$ sudo apt install ninja-build
```

Otherwise, grab the binary from [https://github.com/ninja-build/ninja/releases](https://github.com/ninja-build/ninja/releases) and add it your `PATH`.

### Linux dependencies

If you're building for Linux, you might need a few more dependencies. On Ubuntu, you can install the following packages:

```sh
$ sudo apt install libxrandr-dev libxinerama-dev libx11-dev \
    libxcursor-dev libxi-dev libxext-dev libxcb-shm0-dev libxtst-dev \
    libx11-xcb-dev
```

### Windows dependencies

If you're targeting Windows, you need clang and llvm to cross compile for the MSVC target. Dawn doesn't seem to work with mingw as of writing this.

```sh
$ sudo apt install clang-12 clang-tools-12 llvm-12
```

You'll also need a copy of the Windows SDK headers and libraries. [xwin](https://github.com/Jake-Shadle/xwin) is a super handy tool which can be used to download them on Linux. Install it from source or grab a binary from the releases, then run the following:

```sh
$ xwin splat --include-debug-libs --cache-dir /path/to/sdk
```

The `/path/to/sdk` can be anywhere on your system where you'd like to download the SDK.

### Environment variables

If you're cross-compiling for Windows, you need to set a few environment variables. The easiest thing would be to add them to your shell profile, but I'd recommend using a tool like [direnv](https://github.com/direnv/direnv) to avoid possible interference between projects.

```sh
# If you installed llvm on Ubuntu like above, this should be `/usr/lib/llvm-12`
export LLVM_NATIVE_TOOLCHAIN="/path/to/llvm"
# This should be the path to wherever you downloaded the Windows SDK with xwin
export XWIN_CACHE="/path/to/sdk"
# These variables tell Cargo which C/C++ compiler and linker to use, and where to find
# the Windows SDK
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="\
    -C linker=$LLVM_NATIVE_TOOLCHAIN/bin/lld-link \
    -Lnative=$XWIN_CACHE/splat/crt/lib/x86_64 \
    -Lnative=$XWIN_CACHE/splat/sdk/lib/ucrt/x86_64 \
    -Lnative=$XWIN_CACHE/splat/sdk/lib/um/x86_64"
export CXX_x86_64_pc_windows_msvc="$LLVM_NATIVE_TOOLCHAIN/bin/clang-cl"
export CXXFLAGS_x86_64_pc_windows_msvc="\
    /imsvc $XWIN_CACHE/splat/crt/include \
    /imsvc $XWIN_CACHE/splat/sdk/include/ucrt"
export AR_x86_64_pc_windows_msvc="$LLVM_NATIVE_TOOLCHAIN/bin/llvm-lib"
```

## Building

Make sure to clone wgslsmith recursively to fetch the submodules:

```sh
$ git clone --recursive https://github.com/wgslsmith/wgslsmith
$ cd wgslsmith
```

To build everything, run the following:

```sh
$ ./build.py
```

This will automatically fetch and build dawn, and then build wgslsmith and the harness.

If cross compiling for Windows, you need to instead set the target explicitly:

```sh
$ ./build.py --target x86_64-pc-windows-msvc
```

If you only want to build the harness, you can run:

```sh
$ ./build.py harness [--target <target>]
```

Build output will be in `target/release` (or `cross-target/<target>/release` when cross compiling).

## Installation

To make the `wgslsmith` command available globally, run the following (after building):

```sh
$ ./build.py install [--install-prefix <path>]
```

This will create a symlink to `target/release/wgslsmith`, so you don't need to rerun it every time you rebuild. Use the `--install-prefix` option to specify the directory to install in (defaults to `/usr/local/bin`).
