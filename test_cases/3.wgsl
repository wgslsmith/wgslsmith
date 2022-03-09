// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[119,193,224,67]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}
// Seed: 5318694563514934552

var<private> LOOP_COUNTERS: array<u32, 3>;

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
    a: u32;
    b: i32;
    c: i32;
    d: i32;
    e: bool;
    f: i32;
    g: bool;
};

struct Struct_2 {
    a: u32;
};

struct Struct_3 {
    a: i32;
    b: i32;
    c: vec3<i32>;
    d: bool;
};

struct Struct_4 {
    a: i32;
    b: vec2<bool>;
    c: bool;
    d: i32;
    e: vec4<i32>;
    f: bool;
    g: vec3<bool>;
    h: Struct_2;
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

fn SAFE_PLUS_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y), SAFE_PLUS_u32(a.z, b.z), SAFE_PLUS_u32(a.w, b.w));
}

fn SAFE_MINUS_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z));
}

fn SAFE_MINUS_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z), SAFE_MINUS_i32(a.w, b.w));
}

fn SAFE_TIMES_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y));
}

fn SAFE_TIMES_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y), SAFE_TIMES_i32(a.z, b.z));
}

fn SAFE_TIMES_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y));
}

fn SAFE_TIMES_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y), SAFE_TIMES_u32(a.z, b.z));
}

fn SAFE_DIVIDE_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y));
}

fn SAFE_DIVIDE_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y), SAFE_DIVIDE_i32(a.z, b.z), SAFE_DIVIDE_i32(a.w, b.w));
}

fn SAFE_MOD_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_MOD_i32(a.x, b.x), SAFE_MOD_i32(a.y, b.y), SAFE_MOD_i32(a.z, b.z));
}

fn SAFE_MOD_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_MOD_i32(a.x, b.x), SAFE_MOD_i32(a.y, b.y), SAFE_MOD_i32(a.z, b.z), SAFE_MOD_i32(a.w, b.w));
}

fn SAFE_MOD_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y));
}

fn SAFE_MOD_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z), SAFE_MOD_u32(a.w, b.w));
}

fn func_4() -> bool {
    if ((dot(clamp(clamp(SAFE_TIMES_vec3_i32(vec3<i32>(-530040752, 1423866527, 518776662), vec3<i32>(424960650, -220785211, 482630676)), ~(vec3<i32>(222182268, -937388975, -103657904)), ~(vec3<i32>(-1007794589, -780788255, 2118627442))), (-(vec3<i32>(841162344, 4377234, 231206840))) ^ (-(vec3<i32>(1380442705, -1269236863, -497886104))), min(SAFE_PLUS_vec3_i32(vec3<i32>(1877349379, 301916629, -1487035216), vec3<i32>(1089704903, 956715997, -129630182)), -(vec3<i32>(489688830, 103004013, -1573130748)))), vec3<i32>(dot(min(vec4<i32>(264433819, 923839858, -535724464, -1793743208), vec4<i32>(471606685, 1238913280, 742796322, 1517828088)), -(vec4<i32>(1585009615, -1931054537, 1967905562, -307417320))), abs(min(-1043034809, 538809552)), -(max(-1761302328, -180722709))))) >= ((-(-475954635)) & (SAFE_DIVIDE_i32(1106165386, min(-(-1274788840), dot(vec4<i32>(1108447816, 1662230735, 1233647015, 348350754), vec4<i32>(1937767340, 188627470, -623140384, 226116526))))))) {
        let var_0 = Struct_2(~(~(1681315436u)));
    }
    var var_0 = select(select(clamp(vec3<i32>(1867909793, -389970435, ~(-564251088)), -(abs(vec3<i32>(-1022416679, -975707115, 578562181))), ((vec3<i32>(-1994652554, -1332021685, 2105196970)) & (vec3<i32>(-217134775, -957850331, -1459117066))) >> (SAFE_TIMES_vec3_u32(vec3<u32>(3727606966u, 938782328u, 1594975886u), vec3<u32>(758013360u, 329392158u, 1944919305u)))), SAFE_MINUS_vec3_i32(SAFE_PLUS_vec3_i32(vec3<i32>(-644938729, -719958857, 1944889281), vec3<i32>(470635442, 576937224, -1488141006)), max(~(vec3<i32>(2107485634, 970401275, -803553982)), clamp(vec3<i32>(686242294, 123861080, 1238902283), vec3<i32>(-763933573, -559330792, 643754446), vec3<i32>(1885948059, 1718395764, 1484723468)))), (false) & (select(!(false), false, true))), clamp(~(select(vec3<i32>(1316141451, -1494397752, 1479724412), ~(vec3<i32>(-496373760, -1739034673, 2145512795)), vec3<bool>(false, false, true))), min(vec3<i32>(-(-1979720767), dot(vec2<i32>(1187673519, -220937737), vec2<i32>(884373107, 1423258593)), (2130420585) << (3744021780u)), vec3<i32>(~(1322866910), -757996789, -165626952)), vec3<i32>(SAFE_PLUS_i32((169241050) >> (3145920825u), -489082654), -2130918982, ~(-(-43010997)))), !((all(select(vec2<bool>(true, true), vec2<bool>(true, false), false))) | (false)));
    if ((all(!(!(vec3<bool>(false, true, true))))) && (true)) {
        let var_1 = -2068357097;
    }
    if (any(select(vec4<bool>((dot(vec2<u32>(738299827u, 1564760671u), vec2<u32>(2297236899u, 1867783000u))) < (~(3321229996u)), !(false), (min(var_0.x, 1273035923)) >= (var_0.x), true), select(select(select(vec4<bool>(false, false, true, true), vec4<bool>(false, false, true, false), vec4<bool>(false, true, false, false)), !(vec4<bool>(true, true, false, true)), vec4<bool>(true, true, false, false)), !(!(vec4<bool>(false, true, true, true))), vec4<bool>(!(true), true, true, all(vec3<bool>(true, true, false)))), vec4<bool>(false, false, true, !(select(true, true, false)))))) {
        if ((546162538u) < (SAFE_PLUS_u32(~(min(~(242554061u), 1477061774u)), dot((select(vec4<u32>(2549945292u, 2249674006u, 78680108u, 3793799360u), vec4<u32>(3196513478u, 3432832449u, 789450857u, 2162019684u), true)) | (vec4<u32>(2888967242u, 235072816u, 2864631388u, 1432073792u)), SAFE_PLUS_vec4_u32(SAFE_MOD_vec4_u32(vec4<u32>(1232970627u, 236564321u, 3466769170u, 2934746910u), vec4<u32>(2997930238u, 1798456565u, 2974320981u, 1765366010u)), vec4<u32>(3812332490u, 1772180779u, 908789326u, 3135513795u)))))) {
            let var_1 = Struct_2(~(~(max(2169353750u, clamp(681011125u, 1175151093u, 829234003u)))));
        }
    }
    var var_1 = Struct_1(max(~(abs(abs(516742711u))), (~(~(2568383567u))) << (~(~(1805787323u)))), 302943243, dot(select(select((var_0.yx) >> (vec2<u32>(1632790893u, 3182424185u)), ~(var_0.zy), !(false)), SAFE_DIVIDE_vec2_i32(SAFE_PLUS_vec2_i32(vec2<i32>(var_0.x, var_0.x), var_0.yx), clamp(vec2<i32>(var_0.x, -56008043), var_0.xz, vec2<i32>(-70550675, var_0.x))), (true) | (!(false))), SAFE_TIMES_vec2_i32(-(var_0.xx), -(abs(vec2<i32>(var_0.x, var_0.x))))), -1983035026, any(vec3<bool>(select(!(true), !(true), false), (dot(vec4<u32>(1714758475u, 3215451327u, 346351765u, 1255187946u), vec4<u32>(4191631154u, 410946116u, 809315902u, 1897348974u))) != (3058490512u), (false) | (true))), dot(vec3<i32>(-(SAFE_TIMES_i32(var_0.x, var_0.x)), select(~(var_0.x), ~(var_0.x), (false) || (false)), var_0.x), vec3<i32>(var_0.x, ((1867954441) ^ (-1834999031)) | (var_0.x), dot(-(vec4<i32>(var_0.x, var_0.x, -1881201057, var_0.x)), vec4<i32>(var_0.x, -920366390, -476360658, 1948375205)))), (any(select(vec2<bool>(true, true), select(vec2<bool>(false, false), vec2<bool>(false, false), vec2<bool>(false, false)), true))) || (!(any(select(vec3<bool>(false, false, true), vec3<bool>(false, false, false), true)))));
    {
        var_0.x = var_0.x;
    }
    var_1 = var_1;
    var var_2 = var_0.zx;
    var var_3 = (min(SAFE_TIMES_u32(dot(vec3<u32>(3783202600u, 563534052u, 824179714u), vec3<u32>(var_1.a, var_1.a, 543600837u)), 2651005508u), 1113372047u)) & (var_1.a);
    var var_4 = any(select(!(select(vec2<bool>(false, true), !(vec2<bool>(false, true)), var_1.e)), vec2<bool>(any(vec3<bool>(var_1.e, var_1.e, true)), all(!(vec4<bool>(false, false, false, var_1.g)))), !(select(!(vec2<bool>(var_1.e, true)), vec2<bool>(false, var_1.e), var_1.e))));
    return !(all(select(vec4<bool>(!(true), (1171361671u) > (var_3), true, var_1.e), vec4<bool>(false, var_1.e, all(vec4<bool>(false, false, false, true)), all(vec4<bool>(var_1.g, true, var_1.g, false))), vec4<bool>((2777968037u) == (var_3), var_1.g, (1929264999) != (1574004172), false))));
}

fn func_3() -> vec4<i32> {
    loop {
        if ((LOOP_COUNTERS[0u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[0u] = (LOOP_COUNTERS[0u]) + (1u);
        var var_0 = Struct_3(614517392, -621306362, SAFE_MOD_vec3_i32(vec3<i32>(abs(~(-2128621726)), -(~(-1122948878)), 563499838), -(~(~(vec3<i32>(995633814, -225076716, 633791499))))), func_4());
    }
    loop {
        if ((LOOP_COUNTERS[1u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[1u] = (LOOP_COUNTERS[1u]) + (1u);
        var var_0 = -(SAFE_DIVIDE_i32(~(-87701402), -32375774));
    }
    var var_0 = select(vec4<i32>(45121615, (min(dot(vec3<i32>(41264318, 148187811, 1675989042), vec3<i32>(665096358, 1519540072, 1836948020)), -214290832)) >> (~(1245450070u)), -1904665893, abs(~(SAFE_MOD_i32(-241335697, -552655690)))), min(SAFE_DIVIDE_vec4_i32(vec4<i32>(SAFE_MINUS_i32(831174908, 75071717), (1205981523) ^ (1721923476), -1668619360, -754856617), vec4<i32>(SAFE_DIVIDE_i32(-642275428, 18151264), -(-773383772), SAFE_MOD_i32(-1450091701, 1245652348), -(-2092675727))), vec4<i32>((SAFE_DIVIDE_i32(212085129, 1365095013)) ^ (abs(-884669627)), min(~(-915724456), max(593671391, -1798400243)), max(977891431, -899170632), -1308133498)), select(!(vec4<bool>((true) && (false), func_4(), (-1314582426) == (935531094), !(false))), !(vec4<bool>(!(false), func_4(), (607609876u) < (584956366u), any(vec4<bool>(false, false, false, true)))), select(!(vec4<bool>(true, true, true, false)), select(select(vec4<bool>(true, false, false, true), vec4<bool>(true, true, true, true), vec4<bool>(true, false, false, true)), vec4<bool>(false, false, false, true), select(vec4<bool>(false, false, true, false), vec4<bool>(false, false, true, true), vec4<bool>(true, false, true, false))), (true) | (all(vec4<bool>(true, false, true, true))))));
    let var_1 = Struct_1(~(SAFE_PLUS_u32(max(~(2181186063u), SAFE_TIMES_u32(3107798987u, 3991057350u)), 749665609u)), 597128360, var_0.x, 566810840, true, -32497889, true);
    if (var_1.e) {
        loop {
            if ((LOOP_COUNTERS[2u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[2u] = (LOOP_COUNTERS[2u]) + (1u);
            var_0.x = -1712876595;
        }
    }
    var_0.x = SAFE_DIVIDE_i32(var_1.b, var_1.d);
    return ((max(SAFE_PLUS_vec4_i32(SAFE_MINUS_vec4_i32(var_0, vec4<i32>(var_1.d, 1058996175, var_0.x, var_0.x)), abs(vec4<i32>(1594766505, var_0.x, var_1.f, var_1.f))), vec4<i32>(SAFE_DIVIDE_i32(-1015747256, var_0.x), -1389325106, -(var_0.x), (127741820) >> (var_1.a)))) | (var_0)) | (var_0);
}

fn func_2(arg_0: Struct_1, arg_1: Struct_2) -> bool {
    let var_0 = Struct_1(~(3526119759u), -1301290569, -(-(659993743)), ~(dot(vec4<i32>(-1968111593, 1174531824, (-630468914) ^ (-1481497983), SAFE_MINUS_i32(1835206096, 63727419)), ~(func_3()))), all(select(!(vec4<bool>(true, false, true, false)), select(select(vec4<bool>(false, false, true, true), vec4<bool>(true, false, false, true), true), select(vec4<bool>(false, false, true, false), vec4<bool>(true, true, false, false), true), !(true)), select(!(vec4<bool>(false, true, true, false)), !(vec4<bool>(false, false, false, false)), (false) || (true)))), select(dot(vec4<i32>((-1076399199) | (-436072514), -1158523633, ~(2023345342), SAFE_TIMES_i32(2004876362, 1401517157)), vec4<i32>(select(2006440732, -81069683, false), 422796712, -688037114, 67480290)), clamp(SAFE_MINUS_i32((-1857792249) & (-223792642), SAFE_PLUS_i32(1545897548, -1210484507)), (564816034) << (clamp(4049720239u, 2706360750u, 218199451u)), -((-2074497372) >> (1992873899u))), false), (!((3777546041u) >= (select(873241417u, 1150766292u, false)))) && (any(vec4<bool>((3618581524u) > (4147942751u), false, true, false))));
    let var_1 = (-1020636715) <= (var_0.f);
    var var_2 = ~(~((~(SAFE_PLUS_u32(var_0.a, var_0.a))) >> (min(720881580u, 3809606749u))));
    let var_3 = !(select(vec4<bool>(var_1, var_0.g, func_4(), var_0.e), select(!(vec4<bool>(true, var_0.e, var_1, var_0.e)), !(select(vec4<bool>(false, var_0.e, true, false), vec4<bool>(var_0.g, false, var_0.g, true), vec4<bool>(var_1, var_1, true, true))), !(!(var_0.g))), select(select(!(vec4<bool>(var_1, false, var_0.g, var_1)), select(vec4<bool>(var_1, true, var_1, false), vec4<bool>(var_1, false, var_0.g, false), var_0.g), !(false)), vec4<bool>((2211325891u) > (var_0.a), var_0.g, (var_1) & (var_0.g), false), !(false))));
    var_2 = clamp(select((~((2662127472u) | (var_2))) >> (3748449187u), 3531218304u, !(false)), 3837035278u, ~(abs(~(min(4035632238u, 2624575563u)))));
    return !(all(var_3));
}

fn func_1(arg_0: vec4<i32>) -> vec4<u32> {
    if ((true) == (func_2(Struct_1(1924641986u, ~((1136056318) << (2724404451u)), (~(899511743)) >> (~(1787570642u)), -1024602817, func_2(Struct_1(836906638u, 893936765, -569614061, -839664079, false, -69689867, true), Struct_2(4230499231u)), select(SAFE_MINUS_i32(-106365696, 732684557), SAFE_MINUS_i32(-524780905, -1473695722), true), (-1379864130) < ((-764536951) | (1576904571))), Struct_2(SAFE_DIVIDE_u32(SAFE_MINUS_u32(1084973748u, 111178212u), 2411425937u))))) {
        let var_0 = !(true);
    }
    var var_0 = Struct_4(abs(dot(SAFE_MINUS_vec4_i32(~(vec4<i32>(631881966, 222220811, 2131216793, 378292944)), vec4<i32>(766212572, 1814462838, -837410332, -255510700)), vec4<i32>(~(-628381840), ~(1871035535), select(1919602361, 2116456290, true), -779654179))), select(!(vec2<bool>(!(false), !(false))), select(select(vec2<bool>(false, false), !(vec2<bool>(false, true)), (false) | (false)), select(select(vec2<bool>(false, false), vec2<bool>(true, false), vec2<bool>(true, true)), !(vec2<bool>(false, false)), select(vec2<bool>(false, true), vec2<bool>(false, true), true)), vec2<bool>(!(true), (3643486179u) < (3484784571u))), select(!(vec2<bool>(true, true)), vec2<bool>((53235288) <= (-2100315484), !(true)), select(select(vec2<bool>(true, false), vec2<bool>(false, true), vec2<bool>(true, true)), vec2<bool>(false, true), vec2<bool>(true, true)))), func_2(Struct_1(select(clamp(3220209789u, 3194295475u, 1182684629u), 114459484u, !(true)), SAFE_MINUS_i32(max(1705374044, 162410059), -1818275268), (-(1792689762)) << (min(3220947061u, 4045001015u)), (dot(vec4<i32>(-1418798513, 1548165670, 1287283554, 273551592), vec4<i32>(-1166709485, 724155591, 120042571, 278800477))) << (dot(vec4<u32>(37240810u, 721287108u, 2213223060u, 2469617309u), vec4<u32>(2020481623u, 186624258u, 1414458089u, 1305158626u))), all(!(vec3<bool>(true, true, true))), (2037284425) & (select(1966468366, 1956623059, false)), (any(vec4<bool>(true, false, true, true))) | (false)), Struct_2((~(2203341357u)) >> (1255791964u))), SAFE_MOD_i32(SAFE_TIMES_i32(1024199998, SAFE_MOD_i32(dot(vec2<i32>(1786189357, -1188521554), vec2<i32>(404242098, 1950415823)), -(-427478394))), 1074455605), vec4<i32>(SAFE_DIVIDE_i32((~(658061686)) & (-951019237), dot(~(vec3<i32>(374041057, -1162976585, -1707313733)), ~(vec3<i32>(-2103568462, 1876711073, 1884231296)))), dot(SAFE_PLUS_vec4_i32(~(vec4<i32>(160180441, 579795580, -883136819, 1273984507)), SAFE_DIVIDE_vec4_i32(vec4<i32>(-1019953258, -844168901, 1941156615, -973125626), vec4<i32>(-1487560566, -1166304127, -1805126758, -1040292295))), clamp(SAFE_MOD_vec4_i32(vec4<i32>(-90189754, -294622518, -678698314, -446550772), vec4<i32>(637806084, 1251349134, 1226867007, -309068919)), vec4<i32>(-940295595, 103041335, -1299483239, -347154388), vec4<i32>(-1682880302, -438770095, 42432838, 1319724579))), (clamp(clamp(-1520517180, -1069488864, -1182010037), clamp(1866375134, 2060080617, 802342969), clamp(898942655, -340782438, 161192972))) & (dot((vec4<i32>(-1278566476, 1263838712, 1116170720, 302995189)) << (vec4<u32>(503572291u, 2828138499u, 1280652732u, 3011503799u)), vec4<i32>(715386747, -576122663, -846851453, 1767988837))), clamp(217951436, ~((-2028305122) ^ (-1303098403)), -228810871)), !(!((min(1883435986u, 3317747518u)) > (~(1744737144u)))), !(!(select(vec3<bool>(true, true, true), vec3<bool>(false, false, false), select(vec3<bool>(false, true, true), vec3<bool>(false, true, false), vec3<bool>(true, false, true))))), Struct_2(~(SAFE_DIVIDE_u32(SAFE_MINUS_u32(2691795513u, 130984759u), abs(4120985560u)))));
    var var_1 = abs(vec4<u32>(1755618649u, dot(~((vec2<u32>(615436967u, var_0.h.a)) << (vec2<u32>(var_0.h.a, var_0.h.a))), (SAFE_MOD_vec2_u32(vec2<u32>(3337520479u, 1839640477u), vec2<u32>(var_0.h.a, var_0.h.a))) << (SAFE_TIMES_vec2_u32(vec2<u32>(3594752267u, 2562698475u), vec2<u32>(756928668u, var_0.h.a)))), (~(458771810u)) & (var_0.h.a), select(var_0.h.a, 716193022u, !(true))));
    var var_2 = -(587340341);
    var var_3 = var_0.h;
    var_3 = Struct_2(~(var_1.x));
    var var_4 = var_0.e.zyw;
    if (var_0.g.x) {
        var var_5 = var_1.x;
    }
    var var_5 = Struct_1((~(2922405694u)) ^ (var_1.x), (var_2) << (abs((SAFE_DIVIDE_u32(var_0.h.a, var_1.x)) >> (dot(var_1.yz, var_1.yz)))), 148288900, -(~(max(-(var_0.d), -(var_4.x)))), false, abs((617918739) & (1994478005)), func_4());
    return select(var_1, ~(vec4<u32>(2356152490u, dot(SAFE_PLUS_vec3_u32(vec3<u32>(var_5.a, var_5.a, var_1.x), vec3<u32>(2833884928u, var_5.a, 2642639161u)), min(vec3<u32>(var_1.x, var_1.x, var_0.h.a), vec3<u32>(var_0.h.a, var_5.a, 3206101380u))), var_1.x, max(var_1.x, var_3.a))), select(!(!(select(vec4<bool>(false, var_5.e, var_0.c, true), vec4<bool>(false, var_5.g, true, var_0.c), vec4<bool>(var_5.e, true, var_0.c, false)))), select(select(vec4<bool>(true, true, var_0.g.x, var_5.g), select(vec4<bool>(true, true, false, true), vec4<bool>(var_5.e, false, false, var_5.e), vec4<bool>(var_0.g.x, var_5.g, var_5.g, var_5.e)), select(vec4<bool>(false, var_5.g, true, var_0.b.x), vec4<bool>(var_0.b.x, false, var_0.g.x, true), false)), vec4<bool>(false, var_5.e, all(vec3<bool>(var_0.b.x, var_5.g, false)), var_5.e), (~(var_0.h.a)) > (~(var_0.h.a))), !(all(vec3<bool>(var_0.g.x, false, false)))));
}

@stage(compute)
@workgroup_size(1)
fn main() {
    var var_0 = Struct_2(~(dot(max(select(vec4<u32>(2968564280u, 2929657114u, 966814375u, 165971295u), vec4<u32>(2566330475u, 1176227690u, 4278244027u, 1030012022u), true), func_1(vec4<i32>(1930899984, -865519447, -35716922, 1608471254))), ~(SAFE_MOD_vec4_u32(vec4<u32>(3885526794u, 1855211993u, 4207724798u, 1753057393u), vec4<u32>(43592740u, 3025016727u, 4250477451u, 3737095698u))))));
    let var_1 = !(!(vec4<bool>(any(select(vec2<bool>(true, false), vec2<bool>(true, false), vec2<bool>(true, true))), true, (dot(vec3<i32>(-1600811802, 390562328, 112447490), vec3<i32>(-1954108247, 1748567011, -1424405278))) < ((477619301) ^ (-431274087)), (625097221) != (-(570040240)))));
    var var_2 = vec3<u32>(2213551195u, 1075600019u, 2893546501u);
    var_2.x = 2290889646u;
    let var_3 = !(false);
    output.value = input.value;
    output.value = var_2.x;
}

