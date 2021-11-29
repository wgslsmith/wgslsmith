[[block]]
struct Buffer {
    data: array<u32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;
