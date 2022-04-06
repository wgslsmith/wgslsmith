# Usage

In the simplest case, you can just execute a shader by passing the path to the shader file to the harness executable. If no path is provided, the shader will be read from `stdin`.

```sh
$ harness /path/to/shader.wgsl
```

This works if the shader has no inputs or outputs, but will probably fail otherwise.

More generally, two things are required to execute a shader:

1. The WGSL source code
2. A JSON metadata object describing the shader's resource interface

The metadata will be used to setup the pipeline and required buffers, and optionally to initialise the buffer values.

For example, the following shader contains a uniform buffer and a storage buffer, acting as the shader's inputs and outputs respectively. The shader has a single bind group, with `input` bound to slot 0 and `output` bound to slot 1. Refer to the [WebGPU docs](https://gpuweb.github.io/gpuweb/#pipeline-layout) for more info about resource layouts.

```rust
struct Buffer {
    value: u32,
};

@group(0)
@binding(0)
var<uniform> input: Buffer;

@group(0)
@binding(1)
var<storage, read_write> output: Buffer;

@stage(compute)
@workgroup_size(1)
fn main() {
    output.value = input.value;
}
```

This shader can be described by the following metadata object:

```json
{
  "resources": [
    {
      "kind": "UniformBuffer",
      "group": 0,
      "binding": 0,
      "size": 4,
      "init": [100, 186, 236, 132]
    },
    {
      "kind": "StorageBuffer",
      "group": 0,
      "binding": 1,
      "size": 4,
      "init": null
    }
  ]
}
```

The root object contains a single `resources` field which is an array of resource objects. Each resource object must specify its `group`, `binding` and `size` in bytes, as well as the `kind` which may be one of `UniformBuffer` or `StorageBuffer`. The `init` field can specify a byte array which will be used to initialise the buffer if not null. This is useful for specifying inputs.

By default, when executing a shader with an explicit path, the harness will look for a json file with the same name and parent directory as the shader. For example, given a shader file at `/path/to/shader.wgsl`, the harness will look for the metadata file at `/path/to/shader.json`.

You can also specify the metadata file path explicitly by passing `--metadata /path/to/metadata.json` on the command line, or even specify the json object inline: `--metadata '{"resources": [...]}'`.

## Backend selection

TODO
