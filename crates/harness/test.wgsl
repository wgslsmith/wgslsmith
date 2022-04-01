// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[100,186,236,132]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}

struct Buffer {
    value: u32;
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
