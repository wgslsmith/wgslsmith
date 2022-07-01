# Basic usage

To execute a shader, run:

```sh
$ wgslsmith run /path/to/shader.wgsl
```

You can also supply input data to initialize any unfiform/storage buffers, by writing a json file of the form:

```json
{
  "{group}:{binding}": [1, 2, 3, ...]
}
```

This contains a mapping from the buffer ID to a byte array containing the init data. The values of `group` and `binding` should be the corresponding attribute values in the WGSL program:

```rust
struct Buffer {
    value: u32,
};

@group(0) @binding(0) // <- 0:0
var<uniform> input: Buffer;

@group(0) @binding(1) // <- 0:1
var<storage, read_write> output: Buffer;

@compute
@workgroup_size(1)
fn main() {
    output.value = input.value;
}
```

By default, when executing a shader with an explicit path, the harness will look for a json file with the same name and parent directory as the shader. For example, given a shader file at `/path/to/shader.wgsl`, the harness will look for the inputs file at `/path/to/shader.json`.

You can also specify the inputs file path explicitly by passing `/path/to/inputs.json` as the second positional argument on the command line, or even specify the json object inline: `'{"0:0": [...]}'`.
