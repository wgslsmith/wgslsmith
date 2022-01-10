// Seed: 11914452227262506299

struct Buffer {
    data: array<u32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

[[stage(compute), workgroup_size(1)]]
fn main() {
    {
        var var_0 = 2877481056u;
    }
    let var_0 = vec2<i32>(-(1438992099), ~(-((1930906661) / (-(~(-1891348788))))));
    let var_1 = vec2<i32>(-1562281905, ~(~(-398497245)));
    var var_2 = 918007174;
    if (true) {
        if (true) {
            var_2 = -(-1075723491);
        }
    }
    var_2 = -1016159679;
    let var_3 = ((var_1) % (~(vec2<i32>(2065972074, -2062616943)))) - (~(var_0));
    var var_4 = -((var_2) | (-(~(var_2))));
    var_2 = (var_4) - (var_2);
    var var_5 = vec2<i32>(var_4, 632992769);
    {
        {
            var_2 = var_4;
        }
    }
    let var_6 = ((840502795) % (-624860520)) >= (-(var_4));
    output.data[0u] = ~(~((3623004314u) << (~(1089322513u))));
}
