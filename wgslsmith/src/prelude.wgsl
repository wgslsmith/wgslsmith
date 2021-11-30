[[block]]
struct Buffer {
    data: array<u32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

fn safe_divide_u32(a: u32, b: u32) -> u32 {
    if (b == 0u) {
        return a;
    } else {
        return a / b;
    }
}

fn safe_divide_i32(a: i32, b: i32) -> i32 {
    if (b == 0) {
        return a;
    } else {
        return a / b;
    }
}

fn safe_mod_u32(a: u32, b: u32) -> u32 {
    if (b == 0u) {
        return a;
    } else {
        return a / b;
    }
}

fn safe_mod_i32(a: i32, b: i32) -> i32 {
    if (b == 0) {
        return a;
    } else {
        return a / b;
    }
}
