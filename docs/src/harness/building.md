# Building the harness

```admonish info
Building for Windows is supported by cross compiling from Linux (ideally using [WSL](https://docs.microsoft.com/en-us/windows/wsl/)), because I prefer working in WSL. It shouldn't be too hard to build everything natively on Windows but you're on your own.
```

## Environment variables

There are a few environment variables that you need to set to get everything to build. The easiest thing would be to add them to your shell profile but I'd recommend using a tool like [direnv](https://github.com/direnv/direnv) to manage them.

## Dawn

Before building the harness, you will need to grab and build a copy of [dawn](https://dawn.googlesource.com/dawn). You can use the helper script in [https://github.com/wgslsmith/dawn-build](https://github.com/wgslsmith/dawn-build).

There are a few things you may need to set up first:

### CMake

If you don't already have it:

```sh
$ sudo apt install cmake
```

on Ubuntu or [https://cmake.org/download/](https://cmake.org/download/).

### depot_tools

While you can use cmake to build dawn, you still need to install depot_tools to be able to fetch dawn's dependencies. Detailed instructions for how to do that can be found [here](https://commondatastorage.googleapis.com/chrome-infra-docs/flat/depot_tools/docs/html/depot_tools_tutorial.html#_setting_up). The gist of it is:

```sh
# Clone the depot_tools repo somewhere on your system
$ git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git
# Add it to your PATH
$ export PATH=/path/to/depot_tools:$PATH
```

### Ninja

I'd recommend using the [ninja](https://ninja-build.org/) build system generator with cmake to build dawn. You might be able to use other cmake generators, but they might not produce the right filesystem structure for the outputs which is expected by wgslsmith's build script.

On Ubuntu:

```sh
$ sudo apt install ninja-build
```

Otherwise grab the binary from [https://github.com/ninja-build/ninja/releases](https://github.com/ninja-build/ninja/releases) and add it your PATH.

### Linux dependencies

If you're building for Linux, you might need a few more dependencies. On Ubuntu, you can install the following packages:

```sh
$ sudo apt install libxrandr-dev libxinerama-dev libx11-dev \
    libxcursor-dev libxi-dev libxext-dev libxcb-shm0-dev libxtst-dev \
    libx11-xcb-dev
```

### Windows dependencies

If you're targeting Windows, you need clang and llvm to cross compile for Windows' MSVC target. Dawn doesn't seem to work with mingw as of writing this.

```sh
$ sudo apt install clang-12 clang-tools-12 llvm-12
```

You'll also need a copy of the Windows SDK headers and libraries. [xwin](https://github.com/Jake-Shadle/xwin) is a super handy tool which can be used to download that on Linux. Install it from source or grab a binary from the releases, then run the following:

```sh
$ xwin splat --include-debug-libs --cache-dir /path/to/sdk
```

The `/path/to/sdk` can be anywhere on your system where you'd like to download the SDK.

### Do the build

First, clone the `dawn-build` repo which contains dawn as a submodule as well as a helper script to run the build.

```sh
$ git clone https://github.com/wgslsmith/dawn-build
$ cd dawn-build
```

If building for Windows, you need to set a few environment variables:

```sh
# Instructs cmake to use our custom toolchain file for targeting MSVC
export CMAKE_TOOLCHAIN_FILE="$(pwd)/cmake/WinMsvc.cmake"
# If you installed llvm on Ubuntu like above, this should be `/usr/lib/llvm-12`
export LLVM_NATIVE_TOOLCHAIN="/path/to/llvm"
# This should be the path to wherever you downloaded the Windows SDK with xwin
export XWIN_CACHE="/path/to/sdk"
```

Finally, run the build script:

```sh
$ ./scripts/build
```

That will probably take a while to fetch dependencies and build everything. Once it's done, you should have a `build` folder in the root of the `dawn-build` repo. This should contain, among other things, a `gen` folder with some generated code files (the header files are the important bits) and a `lib` folder with a bunch of static libraries in it.

## Harness

Once you've built the dawn libraries, you can build the harness. First, there's a couple of environment variables to set:

```sh
export DAWN_SRC_DIR="/path/to/dawn-build/dawn"
export DAWN_BUILD_DIR="/path/to/dawn-build/build"
```

If building for Windows, make sure to also set the following variables:

```sh
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="\
    -C linker=/path/to/llvm/bin/lld-link \
    -Lnative=/path/to/sdk/splat/crt/lib/x86_64 \
    -Lnative=/path/to/sdk/splat/sdk/lib/ucrt/x86_64 \
    -Lnative=/path/to/sdk/splat/sdk/lib/um/x86_64"
export CXX_x86_64_pc_windows_msvc="/path/to/llvm/bin/clang-cl"
export CXXFLAGS_x86_64_pc_windows_msvc="\
    /imsvc /path/to/sdk/splat/crt/include \
    /imsvc /path/to/sdk/splat/sdk/include/ucrt"
export AR_x86_64_pc_windows_msvc="/path/to/llvm/bin/llvm-lib"
```

Then you can run:

```sh
$ cargo build -p harness --release
```

to build for the host platform, or:

```sh
$ cargo build -p harness --release --target x86_64-pc-windows-msvc
```

to build explicitly for Windows. Alternatively, you can set:

```sh
export CARGO_BUILD_TARGET="x86_64-pc-windows-msvc"
```

to avoid needing to pass `--target <target>` explicitly on the command line.

The harness executable will be contained in `target/release` (or `target/x86_64-pc-windows-msvc/release` if `CARGO_BUILD_TARGET` was set like above).
