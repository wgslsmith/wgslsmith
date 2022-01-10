// Seed: 3933757910522101610

struct Buffer {
    data: array<u32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

[[stage(compute), workgroup_size(1)]]
fn main() {
    {
        {
            if (false) {
                if (!((((1387881967u) >> (~(1884669894u))) - ((3028040736u) % (82613919u))) != (1398318594u))) {
                    var var_0 = false;
                }
            }
        }
    }
    let var_0 = !(!(!(vec4<bool>(false, false, false, !(true)))));
    output.data[0u] = ~(2945982236u);
}

