# Building wgslsmith

```admonish info
Building for Windows is supported by cross compiling from Linux (ideally using [WSL](https://docs.microsoft.com/en-us/windows/wsl/)). It shouldn't be too hard to build everything natively on Windows, but you're on your own.
```

<!-- toc -->

## Dawn Prerequisites

The test harness and program reduction tools depend on [dawn](https://dawn.googlesource.com/dawn), which has a few prerequisites for building. If you don't want to build those tools, you can skip this bit.

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

## Windows

If cross-compiling for Windows, you'll also need to follow the instructions [here](./cross-compiling.md) to set up the compiler and SDK.

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

This will automatically build dawn, and then build wgslsmith.

You can also disable some features if you don't want to build dawn and wgpu:

```sh
$ ./build.py --no-reducer --no-harness
```

If cross compiling for Windows, you need to set the target explicitly:

```sh
$ ./build.py --target x86_64-pc-windows-msvc
```

It's possible to build the harness as a standalone tool:

```sh
$ ./build.py harness
```

Build output will be in `target/release` (or `cross-target/<target>/release` when cross compiling).

## Installing

To make the `wgslsmith` command available globally, run the following (after building):

```sh
$ ./build.py install [--install-prefix <path>]
```

This will create a symlink to `target/release/wgslsmith`, so you don't need to rerun it every time you rebuild. Use the `--install-prefix` option to specify the directory to install in (defaults to `/usr/local/bin`).

Alternatively you can just put the binary somewhere on your `PATH`.
