// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[100,186,236,132]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}
// Seed: 17620056075937757822

var<private> LOOP_COUNTERS: array<u32, 6>;

let INT_MIN = -2147483648;
let INT_MAX = 2147483647;

let UINT_MIN = 0u;
let UINT_MAX = 4294967295u;

fn SAFE_PLUS_i32(a: i32, b: i32) -> i32 {
    if (b > 0 && a > INT_MAX - b || b < 0 && a < INT_MIN - b) {
        return a;
    } else {
        return a + b;
    }
}

fn SAFE_PLUS_u32(a: u32, b: u32) -> u32 {
    if (b > 0u && a > UINT_MAX - b) {
        return a;
    } else {
        return a + b;
    }
}

fn SAFE_MINUS_i32(a: i32, b: i32) -> i32 {
    if (b < 0 && a > INT_MAX + b || b > 0 && a < INT_MIN + b) {
        return a;
    } else {
        return a - b;
    }
}

fn SAFE_MINUS_u32(a: u32, b: u32) -> u32 {
    if (b < 0u && a > UINT_MAX + b || b > 0u && a < UINT_MIN + b) {
        return a;
    } else {
        return a - b;
    }
}

fn SAFE_TIMES_i32(a: i32, b: i32) -> i32 {
    if (a == -1 && b == INT_MIN || a == INT_MIN && b == -1) {
        return a;
    }

    if (a > INT_MAX / b || a < INT_MIN / b) {
        return a;
    }

    return a * b;
}

fn SAFE_TIMES_u32(a: u32, b: u32) -> u32 {
    if (a > UINT_MAX / b) {
        return a;
    }

    return a * b;
}

fn SAFE_DIVIDE_i32(a: i32, b: i32) -> i32 {
    if (b == 0) {
        return a / 2;
    } else {
        return a / b;
    }
}

fn SAFE_DIVIDE_u32(a: u32, b: u32) -> u32 {
    if (b == 0u) {
        return a / 2u;
    } else {
        return a / b;
    }
}


fn SAFE_MOD_i32(a: i32, b: i32) -> i32 {
    if (b == 0) {
        return a % 2;
    } else {
        return a % b;
    }
}

fn SAFE_MOD_u32(a: u32, b: u32) -> u32 {
    if (b == 0u) {
        return a % 2u;
    } else {
        return a % b;
    }
}

struct Struct_1 {
    a: vec3<bool>;
    b: vec2<bool>;
    c: vec3<u32>;
    d: vec3<bool>;
    e: bool;
};

struct Struct_2 {
    a: i32;
    b: vec2<u32>;
    c: i32;
    d: Struct_1;
    e: Struct_1;
    f: i32;
};

struct Buffer {
    value: u32;
};

@group(0)
@binding(0)
var<uniform> input: Buffer;

@group(0)
@binding(1)
var<storage, read_write> output: Buffer;

fn SAFE_PLUS_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_PLUS_i32(a.x, b.x), SAFE_PLUS_i32(a.y, b.y));
}

fn SAFE_PLUS_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_PLUS_i32(a.x, b.x), SAFE_PLUS_i32(a.y, b.y), SAFE_PLUS_i32(a.z, b.z));
}

fn SAFE_PLUS_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_PLUS_i32(a.x, b.x), SAFE_PLUS_i32(a.y, b.y), SAFE_PLUS_i32(a.z, b.z), SAFE_PLUS_i32(a.w, b.w));
}

fn SAFE_PLUS_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y), SAFE_PLUS_u32(a.z, b.z));
}

fn SAFE_MINUS_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z), SAFE_MINUS_i32(a.w, b.w));
}

fn SAFE_MINUS_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y));
}

fn SAFE_MINUS_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y), SAFE_MINUS_u32(a.z, b.z));
}

fn SAFE_TIMES_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y));
}

fn SAFE_TIMES_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y));
}

fn SAFE_TIMES_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y), SAFE_TIMES_u32(a.z, b.z));
}

fn SAFE_TIMES_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y), SAFE_TIMES_u32(a.z, b.z), SAFE_TIMES_u32(a.w, b.w));
}

fn SAFE_DIVIDE_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y));
}

fn SAFE_DIVIDE_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y), SAFE_DIVIDE_i32(a.z, b.z), SAFE_DIVIDE_i32(a.w, b.w));
}

fn SAFE_DIVIDE_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y));
}

fn SAFE_DIVIDE_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y), SAFE_DIVIDE_u32(a.z, b.z), SAFE_DIVIDE_u32(a.w, b.w));
}

fn SAFE_MOD_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z));
}

fn SAFE_MOD_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z), SAFE_MOD_u32(a.w, b.w));
}

fn func_4(arg_0: vec2<u32>, arg_1: Struct_2) -> vec4<i32> {
    if (all(!(vec4<bool>((!(true)) != ((true) == (true)), !(any(vec4<bool>(true, true, true, true))), (clamp(-1050712659, -1948713739, -1883068008)) >= (SAFE_TIMES_i32(-440167346, -1901776231)), (!(true)) != ((false) | (true)))))) {
        loop {
            if ((LOOP_COUNTERS[2u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[2u] = (LOOP_COUNTERS[2u]) + (1u);
            var var_0 = true;
        }
    }
    if (false) {
        if (true) {
            var var_0 = dot(vec3<u32>(~(3209854573u), abs(clamp(3889534624u, 1553997694u, select(1359111264u, 1003790929u, false))), abs(SAFE_MOD_u32(2181607729u, SAFE_DIVIDE_u32(1785796946u, 3059759470u)))), vec3<u32>(max((clamp(3763452907u, 869153048u, 1392509764u)) | (dot(vec3<u32>(2263600862u, 1766519843u, 2396803131u), vec3<u32>(2854869914u, 483552146u, 3947046602u))), ~(~(839620071u))), 3992440805u, 3381581684u));
        }
    }
    let var_0 = SAFE_DIVIDE_u32(1728508219u, dot(~(~(vec4<u32>(3779707237u, 4123531714u, 1009917302u, 4075434504u))), vec4<u32>(1854225607u, SAFE_TIMES_u32(~(1987339624u), (1319309877u) | (3962835377u)), ((3991212680u) >> (3194560712u)) & ((3286912716u) ^ (1361952041u)), abs(~(1815274182u)))));
    let var_1 = var_0;
    if (!(select(!(!(any(vec2<bool>(false, false)))), false, true))) {
        if (any(select(select(!(!(vec4<bool>(true, false, false, true))), vec4<bool>(all(vec3<bool>(false, true, false)), (true) && (true), false, !(true)), vec4<bool>(!(true), !(false), true, !(false))), select(!(vec4<bool>(false, true, false, true)), select(vec4<bool>(false, false, false, true), !(vec4<bool>(true, true, false, true)), !(true)), vec4<bool>((1841782652) > (1248836664), !(false), all(vec4<bool>(false, true, true, false)), false)), (!(!(false))) | (select(true, false, (false) != (false)))))) {
            let var_2 = ~(~(select(1853591478u, var_1, !((1574050258) == (-104031734)))));
        }
    }
    var var_2 = false;
    if (!(all(select(select(select(vec4<bool>(var_2, true, var_2, false), vec4<bool>(true, var_2, true, var_2), var_2), vec4<bool>(var_2, var_2, true, true), (1368790457u) >= (36269054u)), vec4<bool>(var_2, var_2, any(vec4<bool>(true, var_2, var_2, false)), !(var_2)), !(select(var_2, var_2, false)))))) {
        if (any(select(select(select(!(vec3<bool>(var_2, var_2, var_2)), vec3<bool>(false, false, var_2), all(vec2<bool>(true, true))), !(select(vec3<bool>(false, false, false), vec3<bool>(var_2, var_2, var_2), vec3<bool>(var_2, var_2, true))), !(select(vec3<bool>(var_2, var_2, false), vec3<bool>(false, false, true), vec3<bool>(false, false, false)))), !(!(!(vec3<bool>(var_2, false, false)))), !(vec3<bool>((false) || (true), !(true), var_2))))) {
            var_2 = !((true) && (any(!(select(vec2<bool>(false, var_2), vec2<bool>(false, false), false)))));
        }
    }
    loop {
        if ((LOOP_COUNTERS[3u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[3u] = (LOOP_COUNTERS[3u]) + (1u);
        var var_3 = Struct_2(dot(vec4<i32>((abs(-331677503)) | ((-896674732) << (var_1)), dot(SAFE_PLUS_vec3_i32(vec3<i32>(-659705563, -419560738, 2014789307), vec3<i32>(-1294127521, 1728165387, -440816631)), vec3<i32>(-1379505463, -216134017, -62847805)), ((-703579172) | (-408242955)) | ((-1878569338) << (2115176914u)), -579675585), (vec4<i32>(~(1290026959), abs(1327710082), ~(-475483173), abs(1351272464))) << ((abs(vec4<u32>(282575754u, 973056442u, var_0, 2994287407u))) & (vec4<u32>(var_0, 777776001u, 149440327u, 4223225206u)))), select(~(~(SAFE_DIVIDE_vec2_u32(vec2<u32>(849638614u, var_1), vec2<u32>(3158306197u, var_0)))), ~(~(vec2<u32>(var_1, 1733262859u))), !(select(!(vec2<bool>(true, true)), vec2<bool>(var_2, false), var_2))), dot(-(-(vec4<i32>(2035709623, -645183229, -1612673375, -31848694))), vec4<i32>((SAFE_DIVIDE_i32(-1534101967, 1712326903)) >> (var_0), ~(-472070053), ((-554112904) ^ (1737932550)) ^ (dot(vec3<i32>(-363170665, 959946759, -2001730654), vec3<i32>(-1941133607, -1085906092, -737202280))), min(~(-2056186795), -(-1077308134)))), Struct_1(!(select(vec3<bool>(var_2, var_2, var_2), vec3<bool>(true, var_2, true), select(vec3<bool>(var_2, var_2, var_2), vec3<bool>(false, true, var_2), true))), !(!(vec2<bool>(false, var_2))), (SAFE_MOD_vec3_u32((vec3<u32>(var_0, var_0, 1388096526u)) & (vec3<u32>(386009767u, var_0, 800844891u)), abs(vec3<u32>(var_1, 3039563287u, 2723980456u)))) | ((clamp(vec3<u32>(709602559u, 3212574962u, 2025532331u), vec3<u32>(var_0, var_1, 1366686921u), vec3<u32>(1848598417u, 1214136581u, var_1))) ^ ((vec3<u32>(1722723364u, 2270217138u, 2646826057u)) << (vec3<u32>(3124200645u, 3468153792u, var_0)))), vec3<bool>(!(!(false)), var_2, all(!(vec3<bool>(var_2, var_2, var_2)))), !(!(any(vec2<bool>(var_2, var_2))))), Struct_1(select(vec3<bool>(!(false), all(vec3<bool>(true, var_2, var_2)), any(vec4<bool>(var_2, var_2, var_2, var_2))), !(vec3<bool>(var_2, true, var_2)), true), select(!(!(vec2<bool>(var_2, false))), vec2<bool>((701659849) <= (470537864), (var_2) && (var_2)), vec2<bool>(!(true), !(var_2))), vec3<u32>(1771812561u, SAFE_DIVIDE_u32(526175779u, 361960367u), (1603627477u) << ((2615610982u) ^ (var_1))), vec3<bool>(var_2, var_2, (false) == (true)), var_2), ~(2112238724));
    }
    return clamp(clamp((min(vec4<i32>(-1543197508, 994880560, -297698649, -1225667923), min(vec4<i32>(-648829281, 2131014607, -1584399992, 1603356275), vec4<i32>(-492190521, -1879388446, 1365244888, 1453809471)))) >> (~(max(vec4<u32>(684116405u, var_0, 1237610398u, var_1), vec4<u32>(var_0, var_0, 1982359329u, var_0)))), clamp(select(~(vec4<i32>(907143683, 1375045595, -985902156, -2111128410)), vec4<i32>(-1045164029, -882893428, -883805641, -1384932732), any(vec4<bool>(var_2, false, false, var_2))), select(SAFE_PLUS_vec4_i32(vec4<i32>(927531517, 1612341589, 1733988679, -948348750), vec4<i32>(547961057, -1099009145, -804067463, -1960322185)), vec4<i32>(565329581, 1351749034, -250920303, 1244789455), true), clamp(SAFE_MINUS_vec4_i32(vec4<i32>(468564254, 2113464541, -872428548, -258698157), vec4<i32>(688655970, -262042637, -1260169640, -1614274732)), clamp(vec4<i32>(8899905, 784539259, 1314280081, 851860969), vec4<i32>(249533340, 376988116, -1596747635, 1197187503), vec4<i32>(-1935545014, 2071895989, -1150463158, 1689009269)), abs(vec4<i32>(-967680053, -1102002456, -815612621, 1885928402)))), vec4<i32>(951546056, -(1558064018), ~(-(-1794385311)), SAFE_MINUS_i32(-(1392415760), -(2146502407)))), vec4<i32>(dot(select(~(vec2<i32>(48932983, -2131058736)), SAFE_PLUS_vec2_i32(vec2<i32>(-1994600315, -2143548505), vec2<i32>(173093000, -1220469060)), vec2<bool>(var_2, false)), SAFE_DIVIDE_vec2_i32((vec2<i32>(-1922824869, 1390420237)) ^ (vec2<i32>(-1723937982, 2038648952)), (vec2<i32>(-1653338682, -474137392)) | (vec2<i32>(269897345, -2109707501)))), select(-(dot(vec4<i32>(-401260098, 343709183, -1871982381, 1246033083), vec4<i32>(-1407671785, 1395031303, 149827030, -1875676510))), 1371795495, any(!(vec2<bool>(false, true)))), SAFE_DIVIDE_i32(-(dot(vec2<i32>(832295540, -1611029210), vec2<i32>(-1451254031, 38983500))), dot(vec3<i32>(-1913597196, -1118501311, 2011561195), vec3<i32>(-633348992, -628230589, 127251679))), -(~(-(416418218)))), vec4<i32>((-273276447) | (1276459610), select(clamp(249384808, SAFE_TIMES_i32(-292259254, 1985283202), clamp(-833442077, 1634181237, 882752260)), 1512800250, var_2), select(max(1350356494, (-502027775) >> (var_0)), abs(max(1519463847, 601237259)), any(select(vec3<bool>(var_2, var_2, var_2), vec3<bool>(var_2, var_2, var_2), vec3<bool>(false, var_2, var_2)))), -(~(abs(972602062)))));
}

fn func_3(arg_0: vec3<u32>, arg_1: i32, arg_2: Struct_1, arg_3: vec3<u32>) -> vec4<u32> {
    let var_0 = -(select(~(~(vec4<i32>(-1283352067, 93377299, -1131579234, -1983405853))), SAFE_MINUS_vec4_i32(clamp(abs(vec4<i32>(-1494490507, 2006784551, -1856179837, 367388077)), vec4<i32>(784836994, 681285692, -1244589566, -816375777), func_4(vec2<u32>(930218319u, 2484763177u), Struct_2(237934749, vec2<u32>(2099336990u, 1490371939u), 566283192, Struct_1(vec3<bool>(true, true, false), vec2<bool>(true, false), vec3<u32>(1970548492u, 2533155791u, 2633996140u), vec3<bool>(true, true, true), true), Struct_1(vec3<bool>(true, true, false), vec2<bool>(false, false), vec3<u32>(3734349988u, 3381053577u, 1578156115u), vec3<bool>(false, true, false), false), -1729071088))), SAFE_PLUS_vec4_i32(SAFE_DIVIDE_vec4_i32(vec4<i32>(276541037, -668177371, -107844707, 1626672033), vec4<i32>(492923050, 464114614, 533543703, -1435982500)), vec4<i32>(1469441346, 522953997, -528354925, 1847036265))), !((-(-84830486)) >= ((-1680786891) >> (2222732570u)))));
    let var_1 = select(min(SAFE_MOD_vec3_u32((abs(vec3<u32>(262505764u, 1699968487u, 2707134433u))) | (vec3<u32>(3551970277u, 1974857434u, 2688988902u)), ~((vec3<u32>(2951010614u, 384633011u, 3148912613u)) >> (vec3<u32>(3162113393u, 2487928123u, 292028088u)))), vec3<u32>(dot(vec4<u32>(3899172011u, 1598048483u, 101569752u, 2705483230u), vec4<u32>(839295425u, 2755734701u, 2285688073u, 1780081716u)), select(clamp(2595229923u, 3058007039u, 3643176320u), SAFE_TIMES_u32(564471154u, 3946066445u), !(false)), SAFE_TIMES_u32(362150496u, max(61516872u, 2277652739u)))), (~(~(select(vec3<u32>(3485192972u, 3888823245u, 3121073723u), vec3<u32>(2408598104u, 1516401691u, 3979402486u), vec3<bool>(true, false, true))))) ^ (SAFE_MINUS_vec3_u32(clamp(vec3<u32>(480237486u, 2865995208u, 2322922505u), ~(vec3<u32>(3461692832u, 183654171u, 2106207972u)), abs(vec3<u32>(1572294414u, 1162191058u, 4255871195u))), ~((vec3<u32>(3644713366u, 1270900468u, 3471088639u)) << (vec3<u32>(3035831100u, 3642200126u, 4174864913u))))), !(!(vec3<bool>((true) && (true), !(false), any(vec4<bool>(false, true, true, true))))));
    var var_2 = Struct_2(SAFE_MINUS_i32(var_0.x, -(var_0.x)), var_1.yz, dot(-(min((var_0) >> (vec4<u32>(3636841739u, var_1.x, 4191624612u, 4173365181u)), vec4<i32>(var_0.x, var_0.x, 1126841581, 2076984415))), max(min(clamp(var_0, var_0, var_0), -(var_0)), -(max(var_0, vec4<i32>(765841306, var_0.x, var_0.x, 31611823))))), Struct_1(vec3<bool>(!((-1122121946) <= (229390895)), (false) | (all(vec2<bool>(true, true))), true), !(vec2<bool>(!(true), any(vec3<bool>(true, true, false)))), ~(SAFE_TIMES_vec3_u32(vec3<u32>(var_1.x, 1716288800u, var_1.x), ~(vec3<u32>(var_1.x, 433949129u, var_1.x)))), vec3<bool>(true, false, (true) || ((2586180555u) < (2753476617u))), !(true)), Struct_1(!(!(select(vec3<bool>(false, false, true), vec3<bool>(true, true, true), vec3<bool>(true, true, true)))), vec2<bool>(true, (1813623052u) < (~(var_1.x))), vec3<u32>(~(select(1087802168u, var_1.x, true)), SAFE_MOD_u32((403641745u) | (var_1.x), SAFE_TIMES_u32(var_1.x, 1190837558u)), 147943955u), vec3<bool>(!(!(true)), true, all(select(vec3<bool>(true, true, false), vec3<bool>(false, false, true), false))), (min((3768502578u) | (1237747070u), clamp(var_1.x, 4124717328u, 3157593429u))) < (2654339368u)), SAFE_TIMES_i32(var_0.x, var_0.x));
    let var_3 = var_2.a;
    {
        let var_4 = clamp(~((var_1.x) >> (3485170479u)), 1347830218u, SAFE_TIMES_u32(335956747u, var_1.x));
    }
    let var_4 = select(!(var_2.d.b), var_2.e.a.xy, all(!(var_2.d.a.xy)));
    var var_5 = vec2<i32>(var_3, min(~(SAFE_DIVIDE_i32(SAFE_MINUS_i32(1141805212, var_0.x), dot(var_0.yyx, var_0.wzw))), var_3));
    var_5 = var_0.ww;
    var_2 = Struct_2(SAFE_MOD_i32(SAFE_MINUS_i32(~((var_0.x) & (var_0.x)), 1658691044), ~(var_0.x)), ~(~(vec2<u32>(3667387813u, ~(var_1.x)))), -660855857, Struct_1(select(select(!(vec3<bool>(true, false, true)), var_2.e.d, true), var_2.d.a, !((false) & (true))), var_2.d.b, SAFE_PLUS_vec3_u32(var_1, (vec3<u32>(3631572047u, var_2.e.c.x, 3144577376u)) & (~(var_1))), select(vec3<bool>(all(vec3<bool>(var_4.x, false, true)), !(true), true), vec3<bool>(var_2.d.d.x, false, var_4.x), !(false)), (!(var_4.x)) && (false)), Struct_1(var_2.e.d, !(!(select(var_4, var_4, var_2.e.a.x))), SAFE_MOD_vec3_u32(vec3<u32>(var_1.x, ~(var_2.e.c.x), var_1.x), var_1), vec3<bool>(!(var_2.d.b.x), all(!(var_2.d.d.xz)), all(vec4<bool>(true, var_4.x, false, false))), (max((var_2.d.c.x) ^ (var_2.b.x), dot(vec3<u32>(92462851u, 3160263832u, var_1.x), vec3<u32>(var_2.b.x, var_1.x, 1195903717u)))) != (clamp(1107171521u, 2313806879u, ~(2028433452u)))), var_3);
    var_5.x = -42992650;
    return ((SAFE_DIVIDE_vec4_u32(~(vec4<u32>(var_2.d.c.x, var_2.d.c.x, 1287537171u, 1614985145u)), ~(max(vec4<u32>(var_2.b.x, var_1.x, var_2.e.c.x, 2492180142u), vec4<u32>(var_1.x, 1334524188u, var_2.d.c.x, var_1.x))))) | (~(SAFE_MOD_vec4_u32(vec4<u32>(1164142696u, var_1.x, 2730945218u, 679034699u), ~(vec4<u32>(var_2.d.c.x, var_2.d.c.x, 3513288169u, 2353456927u)))))) & (max(select(select(~(vec4<u32>(1317856694u, 1290968637u, 291870648u, var_1.x)), vec4<u32>(1000154244u, 633693094u, 3001431137u, var_2.d.c.x), vec4<bool>(var_2.d.e, true, var_2.e.b.x, false)), (vec4<u32>(4071920643u, 1030717232u, var_2.e.c.x, 1814445149u)) >> (SAFE_DIVIDE_vec4_u32(vec4<u32>(2864239947u, var_2.e.c.x, var_1.x, 2675927220u), vec4<u32>(var_1.x, 2105616695u, 2034832819u, var_2.d.c.x))), any(vec2<bool>(true, false))), vec4<u32>(dot(vec3<u32>(833203415u, 172366904u, var_1.x), select(vec3<u32>(782092149u, var_1.x, var_1.x), var_1, var_2.d.a)), var_1.x, var_1.x, 1023599189u)));
}

fn func_2(arg_0: vec2<i32>, arg_1: Struct_2, arg_2: u32, arg_3: bool) -> Struct_2 {
    if ((true) || (all(!(vec2<bool>(all(vec2<bool>(true, false)), true))))) {
        if (false) {
            if (select((dot(vec4<u32>((3912298991u) ^ (1508988050u), SAFE_TIMES_u32(2616560238u, 3039602592u), min(2517702170u, 886572466u), 40119174u), select(clamp(vec4<u32>(2383760630u, 3153066827u, 3730888692u, 1983055609u), vec4<u32>(4293375478u, 1202932656u, 407027104u, 1805888982u), vec4<u32>(3967827434u, 4100897816u, 3416340775u, 1967536430u)), func_3(vec3<u32>(2889348008u, 2248463944u, 2154752282u), -1023660318, Struct_1(vec3<bool>(false, true, false), vec2<bool>(true, false), vec3<u32>(4240930762u, 2651115310u, 2495050850u), vec3<bool>(false, true, false), false), vec3<u32>(290605988u, 2184216392u, 3747102959u)), vec4<bool>(true, false, true, true)))) <= (SAFE_DIVIDE_u32(SAFE_TIMES_u32(~(659619026u), 1757560864u), ~(~(119743823u)))), (-1896622414) == ((SAFE_MOD_i32((857934820) | (-765074591), -(131209357))) & (~((-1002502191) << (2914375315u)))), !(((clamp(-1496883343, 1801567111, -391171827)) == (-63334038)) && (false)))) {
            }
        }
    }
    if (true) {
        if ((~(dot(((vec2<u32>(4018038671u, 201420733u)) >> (vec2<u32>(2683450967u, 722772342u))) & (~(vec2<u32>(93020839u, 3185510733u))), ~(SAFE_TIMES_vec2_u32(vec2<u32>(4160724377u, 1574151874u), vec2<u32>(845538119u, 3860946638u)))))) != (SAFE_MINUS_u32(~((SAFE_TIMES_u32(3456414356u, 1395583832u)) ^ (min(3588590205u, 1031861584u))), 675695423u))) {
            loop {
                if ((LOOP_COUNTERS[4u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[4u] = (LOOP_COUNTERS[4u]) + (1u);
                var var_0 = 495394116u;
            }
        }
    }
    let var_0 = Struct_2((abs(min(-55269510, dot(vec3<i32>(-1764386147, 89874383, -30069733), vec3<i32>(-362832082, 463610514, 1274589789))))) << (~(SAFE_MINUS_u32(~(2449371599u), ~(3294809814u)))), vec2<u32>(SAFE_DIVIDE_u32(~((2007270129u) | (1530480946u)), dot(~(vec3<u32>(526128770u, 3205727357u, 622625033u)), SAFE_MINUS_vec3_u32(vec3<u32>(2517304782u, 3883571923u, 2894186232u), vec3<u32>(1369353406u, 2476324242u, 1786928067u)))), SAFE_MINUS_u32(SAFE_PLUS_u32(SAFE_MOD_u32(1348815610u, 2700707008u), (2671269354u) << (1177123971u)), 2787320936u)), -(~(-1192779274)), Struct_1(!(!(select(vec3<bool>(true, false, true), vec3<bool>(true, false, false), false))), vec2<bool>(all(select(vec4<bool>(false, true, false, false), vec4<bool>(true, true, true, false), false)), all(select(vec4<bool>(true, false, false, false), vec4<bool>(true, false, true, false), false))), vec3<u32>(SAFE_TIMES_u32(~(801957583u), SAFE_MINUS_u32(1571329835u, 1999280125u)), SAFE_MOD_u32((2118224923u) >> (1314039406u), SAFE_DIVIDE_u32(142105504u, 1853979438u)), ~(SAFE_MOD_u32(2316265140u, 2747817258u))), !(vec3<bool>(false, false, !(false))), (SAFE_TIMES_i32(clamp(1378370137, -1297140303, -1545548719), SAFE_TIMES_i32(-1808327025, -783143982))) <= (dot((vec3<i32>(-988380697, 735218741, -1621406293)) | (vec3<i32>(-2118305129, -1379143244, 538195770)), abs(vec3<i32>(-287530269, 756865799, 562358313))))), Struct_1(select(vec3<bool>(all(vec4<bool>(false, true, false, true)), all(vec3<bool>(false, true, true)), !(false)), !(vec3<bool>(true, true, true)), false), !(vec2<bool>(false, !(true))), vec3<u32>(dot(vec3<u32>(4104363561u, 4273510442u, 1990727571u), clamp(vec3<u32>(2974439662u, 3254788437u, 269223333u), vec3<u32>(621845379u, 4016929740u, 2530875641u), vec3<u32>(1370464384u, 109395180u, 420205344u))), 2603195217u, 820033429u), vec3<bool>(((2055699609) == (1569206713)) == (false), (false) & (false), (13925653u) > (dot(vec3<u32>(2897999976u, 3684148200u, 1939262160u), vec3<u32>(2750982113u, 426976387u, 698635065u)))), ((select(2117967408u, 1343210012u, false)) < ((716926346u) & (3429571339u))) & (true)), 568522815);
    let var_1 = any(select(!(var_0.d.a.xx), var_0.e.d.zz, any(!(vec4<bool>(var_0.d.a.x, true, var_0.d.a.x, true)))));
    var var_2 = var_0;
    let var_3 = Struct_2(var_0.f, SAFE_DIVIDE_vec2_u32(clamp(max(~(vec2<u32>(var_2.b.x, var_2.b.x)), vec2<u32>(var_2.d.c.x, 2764655555u)), abs(vec2<u32>(var_0.d.c.x, 3382344377u)), vec2<u32>(~(var_0.d.c.x), max(935147182u, var_2.e.c.x))), ~((vec2<u32>(var_0.b.x, 2660406358u)) << (select(var_0.d.c.zy, vec2<u32>(var_2.d.c.x, 226017184u), true)))), var_2.c, Struct_1(var_0.d.d, vec2<bool>((!(var_0.e.b.x)) & (all(var_0.d.a.yz)), false), vec3<u32>(1970128352u, dot(var_2.d.c.yz, vec2<u32>(2913785195u, 4283854667u)), abs(select(var_2.e.c.x, var_2.d.c.x, false))), var_2.d.d, var_2.e.d.x), var_2.d, ~(var_0.a));
    return Struct_2(~(~(var_0.f)), (clamp(var_2.d.c.yx, ~(var_2.d.c.yz), (SAFE_MINUS_vec2_u32(vec2<u32>(var_0.b.x, var_2.e.c.x), var_2.e.c.yz)) >> (abs(var_3.b)))) << (var_0.d.c.xy), SAFE_MINUS_i32(-314301745, select(min(-(var_0.c), 1724280172), -(142260123), (SAFE_DIVIDE_u32(var_0.e.c.x, var_0.b.x)) > (clamp(var_0.b.x, 255989050u, var_3.d.c.x)))), Struct_1(!(select(!(vec3<bool>(true, true, true)), !(var_3.e.a), !(vec3<bool>(true, var_3.d.e, true)))), select(var_0.d.b, vec2<bool>(any(var_0.d.a), false), ((true) | (var_2.d.b.x)) && ((517545462u) != (var_3.b.x))), vec3<u32>(abs(dot(var_2.e.c, vec3<u32>(var_0.b.x, var_2.b.x, 2186807984u))), abs(~(var_0.b.x)), (var_3.d.c.x) ^ (1608621649u)), select(!(var_0.d.d), var_2.e.a, true), (any(!(vec2<bool>(true, true)))) != (var_2.d.e)), var_2.d, select(var_3.a, SAFE_MOD_i32(clamp(min(-1304697594, var_2.f), var_2.c, (var_3.a) & (494938521)), -(var_3.f)), true));
}

fn func_1(arg_0: Struct_1) -> vec2<bool> {
    var var_0 = func_2(-(~(vec2<i32>(SAFE_PLUS_i32(-1088598492, -1450725847), ~(-671299881)))), func_2(vec2<i32>(SAFE_MINUS_i32(161109703, 1883597764), select(SAFE_MOD_i32(358713409, -1313517694), dot(vec4<i32>(1729404143, 1136539052, -227909551, -327124761), vec4<i32>(-278810253, -278967943, 1480608572, -1391257677)), select(true, false, true))), func_2(max((vec2<i32>(-1893986819, -296377427)) >> (vec2<u32>(2263783784u, 2029465708u)), SAFE_DIVIDE_vec2_i32(vec2<i32>(-1884797677, 564872142), vec2<i32>(647505690, -284313678))), Struct_2(SAFE_MINUS_i32(-1742782475, 287884346), vec2<u32>(423411411u, 2064578813u), dot(vec2<i32>(1090619355, 86147296), vec2<i32>(-747976481, -229579674)), Struct_1(vec3<bool>(true, false, false), vec2<bool>(false, false), vec3<u32>(1682925087u, 456044621u, 2737498647u), vec3<bool>(false, true, true), false), Struct_1(vec3<bool>(true, true, false), vec2<bool>(true, false), vec3<u32>(1283084376u, 4137343897u, 1517601758u), vec3<bool>(true, true, true), true), -1183381894), 2964239140u, all(select(vec2<bool>(true, true), vec2<bool>(true, true), vec2<bool>(false, false)))), dot(SAFE_TIMES_vec4_u32((vec4<u32>(1062484841u, 2842129099u, 2220882986u, 2512857548u)) >> (vec4<u32>(1761491098u, 319493772u, 3993959744u, 3462070685u)), max(vec4<u32>(2142005248u, 3015495719u, 1179847358u, 1247635491u), vec4<u32>(2828159169u, 556476841u, 1941567663u, 2080213579u))), SAFE_TIMES_vec4_u32(vec4<u32>(2969642310u, 589625483u, 3373550695u, 1202415477u), SAFE_MOD_vec4_u32(vec4<u32>(3098773151u, 2066275257u, 4182179473u, 3388276133u), vec4<u32>(2690319969u, 841746317u, 1290111420u, 2454459834u)))), (dot(vec4<u32>(3851587580u, 3557395482u, 3578520994u, 3436798902u), clamp(vec4<u32>(3275719378u, 2637070023u, 1533681956u, 1584890591u), vec4<u32>(1540198036u, 2860086288u, 1083140334u, 1558367079u), vec4<u32>(1543326003u, 673374741u, 1839779447u, 3183259089u)))) >= (SAFE_DIVIDE_u32(~(3420991042u), SAFE_DIVIDE_u32(783443257u, 1475188210u)))), ~(clamp((~(844710377u)) >> (3707340285u), (2888859191u) << (~(3321065040u)), 1155903889u)), (((abs(1920143257)) & (dot(vec2<i32>(-1055823500, -994194335), vec2<i32>(-1913891636, 931503206)))) | (SAFE_TIMES_i32(dot(vec2<i32>(1016345657, -1746608378), vec2<i32>(-954719484, -14827406)), 24635578))) >= (~(-129490905)));
    var_0 = var_0;
    let var_1 = var_0.e.c;
    let var_2 = Struct_1(!(!(var_0.e.d)), var_0.e.b, vec3<u32>(~((dot(var_0.e.c, var_1)) & (dot(vec4<u32>(var_1.x, 3563698119u, var_0.b.x, 2544770414u), vec4<u32>(3774046817u, var_1.x, var_0.b.x, var_0.d.c.x)))), SAFE_MOD_u32(var_0.d.c.x, SAFE_MOD_u32(2842700380u, var_0.d.c.x)), var_0.e.c.x), vec3<bool>(var_0.e.d.x, false, var_0.d.a.x), !(!((true) && (var_0.d.e))));
    if ((false) | (true)) {
        if (!((max(var_0.c, var_0.f)) == ((2017500882) ^ (~(-32163981))))) {
            {
                loop {
                    if ((LOOP_COUNTERS[5u]) >= (1u)) {
                        break;
                    }
                    LOOP_COUNTERS[5u] = (LOOP_COUNTERS[5u]) + (1u);
                    var_0 = func_2(SAFE_TIMES_vec2_i32((vec2<i32>(-1135167236, 1870877035)) >> (abs(select(var_1.zz, vec2<u32>(var_0.b.x, var_1.x), vec2<bool>(var_0.d.d.x, var_2.a.x)))), (vec2<i32>(var_0.f, var_0.c)) >> (var_0.e.c.xz)), func_2(-(vec2<i32>((var_0.a) | (var_0.c), (-1265871220) | (var_0.f))), func_2(vec2<i32>((195396637) & (var_0.a), SAFE_MOD_i32(var_0.c, var_0.a)), func_2(select(vec2<i32>(-1761792860, -1799331905), vec2<i32>(var_0.c, -1740327011), var_0.e.b), func_2(vec2<i32>(var_0.c, -1715889483), var_0, 2995741947u, false), var_0.e.c.x, (false) && (var_2.d.x)), dot(SAFE_DIVIDE_vec2_u32(var_1.xx, var_1.yx), var_0.e.c.xy), true), var_0.b.x, var_2.b.x), (var_2.c.x) ^ (abs(~(1117414785u))), any(!(var_2.d.yy)));
                }
            }
        }
    }
    if (any(select(select(select(vec4<bool>(var_0.d.e, var_2.e, true, var_0.d.e), select(vec4<bool>(false, true, var_2.b.x, var_2.a.x), vec4<bool>(true, var_2.b.x, true, false), vec4<bool>(var_2.d.x, var_2.d.x, var_2.e, var_0.e.b.x)), false), !(!(vec4<bool>(var_0.e.b.x, false, var_2.d.x, var_0.e.a.x))), !(!(var_2.e))), vec4<bool>(((var_0.d.c.x) != (var_1.x)) && ((true) || (var_2.b.x)), var_2.b.x, all(var_2.d.yx), !(all(vec4<bool>(var_0.e.a.x, true, var_2.e, false)))), select(vec4<bool>(all(vec4<bool>(var_2.a.x, var_0.d.e, true, true)), any(vec4<bool>(false, var_0.d.b.x, true, true)), var_0.e.d.x, !(var_0.e.b.x)), select(!(vec4<bool>(true, false, false, true)), !(vec4<bool>(var_2.e, var_0.d.d.x, true, false)), vec4<bool>(var_2.b.x, var_2.a.x, var_0.e.a.x, true)), !(vec4<bool>(true, var_2.a.x, var_2.e, false)))))) {
        let var_3 = var_0;
    }
    return vec2<bool>((!(var_2.e)) && (!(var_0.d.a.x)), true);
}

@stage(compute)
@workgroup_size(1)
fn main() {
    var var_0 = select(select(select(vec2<bool>((3537901723u) <= (3001937978u), any(vec4<bool>(true, false, false, true))), select(vec2<bool>(false, false), func_1(Struct_1(vec3<bool>(false, true, false), vec2<bool>(false, true), vec3<u32>(3655779081u, 2295624799u, 2801188800u), vec3<bool>(false, true, true), false)), true), vec2<bool>(all(vec3<bool>(true, true, false)), true)), select(vec2<bool>(!(true), all(vec2<bool>(true, true))), select(vec2<bool>(true, true), vec2<bool>(false, true), func_1(Struct_1(vec3<bool>(false, true, true), vec2<bool>(false, false), vec3<u32>(2886060187u, 1974476258u, 1661838305u), vec3<bool>(true, true, false), false))), !(vec2<bool>(false, false))), vec2<bool>(false, !(!(false)))), !(!(vec2<bool>((true) | (false), !(false)))), all(vec3<bool>(select((false) && (true), (1337496626u) < (3748399812u), !(true)), (-(-2094764321)) > (clamp(-476259505, 512480949, -1765601141)), false)));
    let var_1 = !(!(!(((true) & (false)) || (var_0.x))));
    if ((all(!(var_0))) && ((var_1) && (true))) {
        if ((701245960u) <= (clamp(2112053011u, ~(533112908u), SAFE_MOD_u32(~((1799675682u) >> (441241019u)), SAFE_MINUS_u32(min(3901727484u, 3867834541u), min(3214112991u, 3816860372u)))))) {
            var var_2 = SAFE_MINUS_u32((min(2021326231u, ~((1056631059u) ^ (2095466283u)))) ^ (~((3182007920u) & (3218156101u))), 2715901736u);
        }
    }
    var var_2 = Struct_1(vec3<bool>(false, false, any(!(!(vec3<bool>(var_1, var_0.x, var_1))))), !(vec2<bool>(var_0.x, any(vec2<bool>(false, var_0.x)))), (~(~(vec3<u32>(2364412897u, 6544561u, 1402793176u)))) ^ (~(~(~(vec3<u32>(941797132u, 1856527791u, 3287937032u))))), select(vec3<bool>(var_0.x, (dot(vec4<u32>(3430855316u, 1379280870u, 2079795951u, 2078848948u), vec4<u32>(3698305737u, 3945110804u, 2279565802u, 670650141u))) <= (1255483847u), true), vec3<bool>(var_1, (false) | ((-862814724) > (344451281)), !(any(vec4<bool>(false, var_0.x, var_1, true)))), !(false)), false);
    var_2 = var_2;
    loop {
        if ((LOOP_COUNTERS[0u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[0u] = (LOOP_COUNTERS[0u]) + (1u);
        var var_3 = Struct_2(-1354601993, select((clamp(max(var_2.c.xz, vec2<u32>(3758616488u, var_2.c.x)), ~(var_2.c.yz), vec2<u32>(var_2.c.x, var_2.c.x))) ^ (clamp(var_2.c.yz, (var_2.c.zz) & (vec2<u32>(1481925813u, var_2.c.x)), ~(var_2.c.yy))), var_2.c.yx, (~(-2005329351)) > ((-594279451) | (1760679773))), 993038876, var_2, Struct_1(vec3<bool>(false, var_1, any(select(vec3<bool>(var_2.e, var_2.e, false), vec3<bool>(true, false, false), var_1))), !(vec2<bool>(any(var_0), var_2.d.x)), ~(min(~(vec3<u32>(3273133315u, var_2.c.x, var_2.c.x)), abs(vec3<u32>(var_2.c.x, 567235072u, var_2.c.x)))), vec3<bool>(all(select(var_2.a, vec3<bool>(true, var_0.x, false), vec3<bool>(false, false, true))), any(var_2.a), (min(-1216326428, -1644961968)) > (-308129489)), any(func_1(var_2))), dot(vec3<i32>(1238740659, -(1272802419), 1340203462), vec3<i32>(dot(SAFE_TIMES_vec2_i32(vec2<i32>(-1768399035, -1021704942), vec2<i32>(930657405, -1503977840)), -(vec2<i32>(-1073463655, -673179174))), -(min(-1115563344, 1789594517)), dot(-(vec4<i32>(-1666736798, -1089387050, 344309449, -1503857000)), vec4<i32>(1613154920, -1969079645, -1950300088, 345829489)))));
    }
    loop {
        if ((LOOP_COUNTERS[1u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[1u] = (LOOP_COUNTERS[1u]) + (1u);
        var var_3 = func_2(min(vec2<i32>(select(-1602598233, -15264548, false), -436013228), -(SAFE_PLUS_vec2_i32(SAFE_PLUS_vec2_i32(vec2<i32>(885999822, 1355687577), vec2<i32>(-550178381, 2056201323)), vec2<i32>(1592910514, 1686865842)))), Struct_2(-(select(SAFE_TIMES_i32(864728715, -1934896985), ~(-2132698601), (-838358152) <= (2051784000))), var_2.c.xx, 1101555927, var_2, Struct_1(select(vec3<bool>(var_0.x, false, false), var_2.d, !(var_1)), !(func_1(Struct_1(vec3<bool>(var_2.d.x, var_1, false), vec2<bool>(false, var_2.e), vec3<u32>(2793723155u, var_2.c.x, var_2.c.x), vec3<bool>(var_1, var_1, var_2.d.x), true))), select(~(var_2.c), vec3<u32>(2732941984u, 1042546991u, var_2.c.x), !(true)), !(var_2.a), false), ~(abs(dot(vec2<i32>(-441045181, 1545206313), vec2<i32>(-616861865, 453496062))))), 847683038u, !((!((-1877590704) < (-1884192108))) && (var_1)));
    }
    output.value = input.value;
    output.value = var_2.c.x;
}

