// Seed: 16522641491399582700

struct Buffer {
    data: array<u32>;
};

@group(0)
@binding(0)
var<storage, read_write> output: Buffer;

@stage(compute)
@workgroup_size(1)
fn main() {
    {
        if (!(!((!((true) & (true))) | ((-(-53725130)) == (~(-958936844)))))) {
            let var_0 = 2697232845u;
        }
    }
    {
        if ((~((368319850u) * (~(~(1191074379u))))) > ((~(((1761099246u) * (4064441978u)) >> (504003074u))) << (~(3634537706u)))) {
            var var_0 = vec4<i32>(2034176869, (~(-2049907504)) + (~((-1368158069) & (960065732))), ~(-1681321143), -507618193);
        }
    }
    var var_0 = ~(-(~(vec3<i32>(~(-156013087), 1618962741, 1103555348))));
    if ((((-223360983) << (~((3908557796u) + (1562075981u)))) >= ((((-990950853) & (900408222)) << (1465930706u)) * (1567584552))) | ((!(true)) && ((true) || (((true) || (false)) != (!(true)))))) {
        let var_1 = (~((vec4<u32>((~(2426876117u)) << (~(3132558408u)), ((524609238u) >> (1174971662u)) + ((1944531023u) + (2532130817u)), 3537825187u, ((2283357523u) - (3570541870u)) >> (3654795350u))) | ((~(vec4<u32>(1019875859u, 2398648318u, 2340032909u, 3460618504u))) ^ (~(vec4<u32>(566434961u, 2645728325u, 817718353u, 698163709u)))))) << (vec4<u32>((1722414065u) ^ (~((~(2823713674u)) / (~(2696271792u)))), ~(~((~(2521112820u)) >> ((168024368u) - (3885884106u)))), 4269250743u, ~(~(2288874372u))));
    }
    var_0 = (var_0) ^ (((-((vec3<i32>(277600113, 124788365, -1654770255)) & (vec3<i32>(2084414903, 1332696320, 596532897)))) >> (~(vec3<u32>(~(1723915151u), (3717996403u) - (1253220191u), 981526985u)))) << (~(vec3<u32>(~(~(1804849928u)), 3578212422u, 1656527638u))));
    {
        var var_1 = var_0;
    }
    {
        {
            if (false) {
                let var_1 = (!((331019648) < (-(-(-1781747270))))) && (true);
            }
        }
    }
    var var_1 = var_0;
    var_0 = vec3<i32>(1739906175, ~(-((((1396494543) | (2144343107)) >> (~(1636559872u))) + ((-2034490911) | (~(1886730911))))), 1932290691);
    output.data[0u] = ~(~((2614829996u) + (~((2109115467u) & (3286866300u)))));
}
