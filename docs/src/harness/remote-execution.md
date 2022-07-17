# Remote execution

The harness supports remote execution over a TCP connection, to enable running shaders on a separate device such as an Android smartphone or across a VM boundary (e.g. WSL to Windows host). This is done by running the harness as a server, which receives requests to execute a shader against a set of configurations.

To enable this, wgslsmith should be compiled for both the target machine as well as the client. It is also possible to compile the harness as a standalone tool to run on the execution server (without the other fuzzing tools) as described [here](../building/index.md#building).

```admonish warning
You must ensure that the the client and server are both compiled from the same git commit. No stability guarantees are currently made for the communication protocol, so there may be breaking changes between versions.
```

Use the `serve` subcommand to start the server.

```sh
$ wgslsmith harness serve -a 0.0.0.0:1234
```

The `remote` subcommand can then be used to interact with the server. The command syntax is identical to local harness usage.

```sh
$ wgslsmith remote 192.168.1.23:1234 list
$ wgslsmith remote 192.168.1.23:1234 run path/to/shader.wgsl
```

Note that the first argument to `remote` is the address of the server to connect to. For convenience, wgslsmith allows you to create friendly names for addresses and to set a default address. This is done through a configuration file (open it in your editor by running `wgslsmith config`).

```toml
# wgslsmith.toml

# Define a remote server called 'android-phone'
[remote.android-phone]
address = "192.168.1.23:1234"

[harness]
remote = "android-phone" # Set 'android-phone' as the default remote
```

This will allow you to connect to the remote using:

```sh
$ wgslsmith remote android-phone run shader.wgsl
# or
$ wgslsmith remote run shader.wgsl
```
