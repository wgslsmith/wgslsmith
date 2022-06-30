# Validation Server

<!-- toc -->

## Overview

The FXC and Metal compilers can be used to validate HLSL and MSL shaders. Unfortunately neither of these tools are actually available for Linux, but it's possible to run them using [wine](https://www.winehq.org/).

Running with wine has a startup cost of a few seconds, which can make program reduction significantly slower. We get around this by using a server for validating shaders. This is packaged along with a Wine installation in a docker container.

## Obtaining the compilers

### FXC

The FXC dll is already included in the `tools` subdirectory of the wgslsmith repository.

### Metal

You can download the Metal Developer Tools for Windows from [here](https://developer.apple.com/download/all/?q=metal). You'll need an Apple ID to get access. Copy the installer to `tools/Metal_Developer_Tools.exe`.

## Toolchain setup

The validation server needs to be compiled for Windows. Following the instructions [here](../building/cross-compiling.md#windows) to setup a cross-compilation environment on Linux.

## Usage

Start the validation server using the following command. This will compile the server and then build and start a docker container containing wine and the compiler tools.

```sh
$ ./start-validation-server.sh
```

The default server port is 9123. Add this to your wgslsmith config file (using the `wgslsmith config` command).

```toml
[validator]
server = "localhost:9123"
```
