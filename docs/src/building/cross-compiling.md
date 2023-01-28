# Cross-compiling

<!-- toc -->

## Windows

Windows is supported through cross-compilation using llvm's msvc target.

Add the msvc target to your rust toolchain with rustup:

```sh
$ rustup target add x86_64-pc-windows-msvc
```

Install the clang and llvm packages (on Ubuntu):

```sh
$ sudo apt install clang-14 clang-tools-14 llvm-14 lld-14
```

You'll also need a copy of the Windows SDK headers and libraries. [xwin](https://github.com/Jake-Shadle/xwin) is a super handy tool which can be used to download them on Linux. Install it from source or grab a binary from the releases, then run the following:

```sh
$ xwin splat --include-debug-libs --output /path/to/sdk
```

The `/path/to/sdk` can be anywhere on your system where you'd like to download the SDK.

Also, make sure to set these environment variables:

```sh
# If you installed llvm on Ubuntu like above, this should be `/usr/lib/llvm-14`
export LLVM_NATIVE_TOOLCHAIN="/path/to/llvm"
# This should be the path to wherever you downloaded the Windows SDK with xwin
export XWIN_CACHE="/path/to/sdk"
```
