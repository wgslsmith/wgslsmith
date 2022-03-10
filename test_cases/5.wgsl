// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[93,178,12,132]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}
// Seed: 14183108311046347537

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
    a: vec2<i32>;
};

struct Struct_2 {
    a: u32;
    b: Struct_1;
    c: Struct_1;
    d: vec3<bool>;
    e: vec3<i32>;
    f: vec3<u32>;
    g: i32;
    h: vec4<bool>;
};

struct Struct_3 {
    a: u32;
    b: vec4<u32>;
    c: vec2<i32>;
    d: Struct_1;
    e: vec4<i32>;
    f: bool;
    g: bool;
    h: i32;
    i: i32;
    j: Struct_1;
};

struct Struct_4 {
    a: bool;
    b: Struct_3;
    c: vec4<u32>;
    d: u32;
};

struct Struct_5 {
    a: Struct_3;
    b: Struct_2;
    c: vec4<bool>;
    d: vec4<i32>;
    e: i32;
};

struct Struct_6 {
    a: Struct_2;
    b: Struct_4;
    c: vec3<u32>;
    d: Struct_5;
    e: u32;
    f: u32;
    g: bool;
    h: vec4<u32>;
    i: Struct_3;
    j: u32;
};

struct Struct_7 {
    a: bool;
    b: vec3<u32>;
    c: Struct_4;
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

fn SAFE_MINUS_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y));
}

fn SAFE_MINUS_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z));
}

fn SAFE_MINUS_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z), SAFE_MINUS_i32(a.w, b.w));
}

fn SAFE_MINUS_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y), SAFE_MINUS_u32(a.z, b.z));
}

fn SAFE_MINUS_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y), SAFE_MINUS_u32(a.z, b.z), SAFE_MINUS_u32(a.w, b.w));
}

fn SAFE_TIMES_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y));
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

fn SAFE_DIVIDE_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y), SAFE_DIVIDE_u32(a.z, b.z));
}

fn SAFE_DIVIDE_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y), SAFE_DIVIDE_u32(a.z, b.z), SAFE_DIVIDE_u32(a.w, b.w));
}

fn SAFE_MOD_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_MOD_i32(a.x, b.x), SAFE_MOD_i32(a.y, b.y));
}

fn SAFE_MOD_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_MOD_i32(a.x, b.x), SAFE_MOD_i32(a.y, b.y), SAFE_MOD_i32(a.z, b.z));
}

fn SAFE_MOD_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z));
}

fn SAFE_MOD_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z), SAFE_MOD_u32(a.w, b.w));
}

fn func_4(arg_0: Struct_6) -> i32 {
    let var_0 = (select(~(min(~(vec3<i32>(2090784101, 225150307, 1946858580)), min(vec3<i32>(1962942404, -1693172897, 2038273483), vec3<i32>(-325596130, 958893650, -238360276)))), select(clamp((vec3<i32>(-55397857, 1735576605, -2037067169)) << (vec3<u32>(2986067011u, 772552990u, 1735474135u)), clamp(vec3<i32>(1967807017, -1248408468, -40647110), vec3<i32>(-762198504, -1008029597, 1019548697), vec3<i32>(1844714948, -1806682941, -42095820)), max(vec3<i32>(-399489519, 191941052, -628633085), vec3<i32>(2886838, -1117963667, 1330838292))), ~(vec3<i32>(-936828067, -1823920656, -83228767)), all(!(vec4<bool>(false, false, false, false)))), any(vec2<bool>(!(false), !(true))))) << (~(select(abs(vec3<u32>(1948720139u, 2257243155u, 3317009759u)), clamp(~(vec3<u32>(1281562392u, 253876051u, 4285079276u)), select(vec3<u32>(56989672u, 936651448u, 757153570u), vec3<u32>(2826453751u, 542960843u, 3994004902u), vec3<bool>(false, true, false)), ~(vec3<u32>(3399367803u, 3144067264u, 3308754097u))), !(all(vec2<bool>(true, true))))));
    let var_1 = SAFE_MINUS_vec4_u32(max(~(vec4<u32>(~(3169417012u), 1039011264u, ~(1875742943u), ~(3779828685u))), vec4<u32>(SAFE_PLUS_u32(SAFE_MOD_u32(201613434u, 1818958590u), 3829396356u), max(~(2840011967u), 1551423793u), (~(3530630655u)) << (1066470411u), (1003894182u) ^ (~(2952715081u)))), select(select(~(~(vec4<u32>(436408605u, 455208474u, 1554427292u, 640609987u))), ~((vec4<u32>(3138984696u, 2799700179u, 1665424334u, 604409702u)) & (vec4<u32>(2075263650u, 2083933039u, 1341644057u, 3161533092u))), all(vec2<bool>(true, true))), abs(abs(~(vec4<u32>(1407239132u, 2841381302u, 848891108u, 3260838844u)))), !(true)));
    let var_2 = Struct_7(!(true), SAFE_MINUS_vec3_u32((((vec3<u32>(var_1.x, 1727207891u, var_1.x)) << (vec3<u32>(var_1.x, 4145732553u, 3164248431u))) & (var_1.zyy)) ^ (max(select(vec3<u32>(var_1.x, var_1.x, 3671801450u), var_1.zyw, true), ~(var_1.wxw))), ~(~(min(vec3<u32>(var_1.x, 1149325834u, var_1.x), vec3<u32>(var_1.x, var_1.x, 2781581882u))))), Struct_4(any(!(select(vec2<bool>(false, false), vec2<bool>(false, false), vec2<bool>(true, true)))), Struct_3(SAFE_MOD_u32(max(3406893913u, var_1.x), ~(var_1.x)), select(max(vec4<u32>(1444182666u, 1223125749u, var_1.x, 878299792u), vec4<u32>(589740766u, var_1.x, var_1.x, 1265601683u)), ~(var_1), all(vec3<bool>(true, true, false))), abs(SAFE_MINUS_vec2_i32(var_0.yy, var_0.xz)), Struct_1(-(vec2<i32>(-1624055941, 1029209741))), (~(vec4<i32>(-2093599555, 1644905535, var_0.x, -960555520))) ^ ((vec4<i32>(-302677950, var_0.x, 979338251, var_0.x)) << (var_1)), any(!(vec4<bool>(true, false, true, false))), !(true), -1353553020, -(SAFE_TIMES_i32(var_0.x, 213505243)), Struct_1(vec2<i32>(-281663882, var_0.x))), ~(SAFE_MOD_vec4_u32(var_1, var_1)), ~(var_1.x)));
    let var_3 = SAFE_TIMES_vec4_u32(SAFE_TIMES_vec4_u32(vec4<u32>(dot(clamp(vec2<u32>(277579866u, var_1.x), vec2<u32>(var_2.b.x, 2965768729u), vec2<u32>(var_2.c.c.x, 86242610u)), var_1.ww), 2322404088u, 3440355239u, select(dot(var_1.zw, vec2<u32>(1502264109u, 2014605666u)), var_2.c.d, true)), ~(vec4<u32>(~(var_1.x), ~(var_2.b.x), abs(3587109798u), abs(3047884936u)))), vec4<u32>(var_2.c.c.x, SAFE_TIMES_u32((~(2927636271u)) | (abs(43597877u)), var_2.c.d), 211411583u, dot(vec2<u32>(abs(1402016738u), dot(vec3<u32>(2847513569u, var_1.x, var_1.x), vec3<u32>(var_1.x, var_1.x, var_1.x))), ~(var_1.yx))));
    let var_4 = 1656870726u;
    return ~(var_2.c.b.j.a.x);
}

fn func_3(arg_0: Struct_4, arg_1: bool) -> i32 {
    loop {
        if ((LOOP_COUNTERS[1u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[1u] = (LOOP_COUNTERS[1u]) + (1u);
        var var_0 = (SAFE_TIMES_i32(SAFE_MOD_i32(~(SAFE_MINUS_i32(362278532, -235472817)), -1003652708), -(abs(SAFE_PLUS_i32(2049548416, 1749839851))))) ^ (~(func_4(Struct_6(Struct_2(900352489u, Struct_1(vec2<i32>(-808154026, -1918291577)), Struct_1(vec2<i32>(1317579591, 1368380363)), vec3<bool>(false, true, false), vec3<i32>(-1939302981, 391549762, -1353446205), vec3<u32>(2443434176u, 2802692646u, 170134740u), 1359320908, vec4<bool>(false, true, true, false)), Struct_4(false, Struct_3(3314129486u, vec4<u32>(1735206190u, 299216328u, 2111881294u, 1681785850u), vec2<i32>(-1347539524, -824064352), Struct_1(vec2<i32>(-2000621900, 3831905)), vec4<i32>(1059308379, 1784977996, -45531368, -1768284255), true, true, -915823109, -1611252567, Struct_1(vec2<i32>(-817132522, -1174881701))), vec4<u32>(3250075541u, 3455599253u, 1901981293u, 1496378542u), 2013286509u), vec3<u32>(3810535225u, 3755691667u, 2874505425u), Struct_5(Struct_3(708219433u, vec4<u32>(1950768593u, 698827866u, 1368085995u, 2709094585u), vec2<i32>(1063108095, -1244613910), Struct_1(vec2<i32>(305826715, -1340818423)), vec4<i32>(601894671, 2072649661, -684464997, -1367850087), false, false, -287741703, 1422533982, Struct_1(vec2<i32>(-212182220, -184870782))), Struct_2(1303711848u, Struct_1(vec2<i32>(-1041088624, 251864480)), Struct_1(vec2<i32>(-820856753, -734146290)), vec3<bool>(false, false, true), vec3<i32>(1204941225, -1384221192, 395669496), vec3<u32>(382649926u, 451307460u, 2580510243u), -1856715792, vec4<bool>(true, true, true, false)), vec4<bool>(true, true, false, true), vec4<i32>(-712764439, -633299898, -20168798, 248659664), -923256060), ~(3133531157u), ~(903258651u), (false) || (false), ~(vec4<u32>(3443947524u, 2496833789u, 1127043873u, 958723846u)), Struct_3(870968673u, vec4<u32>(4145513431u, 481084630u, 923471262u, 4120693224u), vec2<i32>(1964886220, -770093915), Struct_1(vec2<i32>(1605017981, -1346189616)), vec4<i32>(-1141959782, 1315359190, 1530227743, -450826329), false, false, 2067289341, 1778036412, Struct_1(vec2<i32>(1023017223, 948437280))), ~(2131527955u)))));
    }
    var var_0 = dot(vec3<i32>(max(clamp(117854962, 1600283558, 792972937), ~(-(-382645608))), 819459467, -(max(-714725530, 477328856))), min(SAFE_MOD_vec3_i32(~(~(vec3<i32>(-250359444, 1642800883, -1249451535))), min(max(vec3<i32>(-174960659, 763836668, 188414572), vec3<i32>(66241659, 66284315, 470680531)), -(vec3<i32>(-1376417754, -2084835696, 660875198)))), clamp(~(SAFE_MINUS_vec3_i32(vec3<i32>(-1561258892, 386513601, -1507151369), vec3<i32>(-1378438017, -1337467356, 879462372))), SAFE_MOD_vec3_i32(select(vec3<i32>(1812134003, -231193655, 758439244), vec3<i32>(-1973485392, 1979252281, 1561028229), true), min(vec3<i32>(1930987053, 1578901070, 478982340), vec3<i32>(2006106232, 1972858813, 907499491))), vec3<i32>(-(307000365), -540826023, -(-1851507510)))));
    if (false) {
        let var_1 = vec2<bool>(select(false, false, (-(SAFE_TIMES_i32(var_0, 1906214474))) > (var_0)), (!(any(!(vec4<bool>(false, true, true, true))))) == (true));
    }
    var_0 = -2073919853;
    var_0 = -1387788489;
    if ((true) | (any(select(!(select(vec3<bool>(false, false, false), vec3<bool>(true, true, false), false)), !(!(vec3<bool>(true, true, false))), true)))) {
        var var_1 = Struct_3(~(clamp(SAFE_TIMES_u32(max(706970912u, 90922169u), 491561188u), (clamp(1839614235u, 651153461u, 103489256u)) ^ (clamp(1410954019u, 168515101u, 1187998064u)), select(433467026u, (1995113486u) >> (3523623741u), !(true)))), vec4<u32>(~(1082380883u), SAFE_MINUS_u32(~(SAFE_MINUS_u32(4233743407u, 1949849697u)), ~(dot(vec4<u32>(4003615255u, 1748323988u, 1772853406u, 1801051206u), vec4<u32>(4573006u, 2108391131u, 1014085877u, 1110878344u)))), ((SAFE_DIVIDE_u32(4061544842u, 2625646953u)) ^ (~(1208205612u))) & (clamp(max(1513164821u, 1058867767u), select(3477326556u, 2416717882u, false), SAFE_PLUS_u32(838502675u, 2786931424u))), ~(~((172906452u) | (3276006181u)))), vec2<i32>(-((~(var_0)) << (~(913405466u))), -797229905), Struct_1((vec2<i32>(~(var_0), var_0)) >> (max(vec2<u32>(49367245u, 3867537663u), vec2<u32>(223459631u, 4094814478u)))), -(abs((abs(vec4<i32>(1457116743, 593938681, 162755715, -1148854689))) >> (min(vec4<u32>(1007635500u, 3770065984u, 955182254u, 3189560854u), vec4<u32>(2755629782u, 3096159589u, 3580033939u, 795384339u))))), all(select(vec4<bool>(false, select(true, false, true), all(vec4<bool>(false, true, true, true)), (3155369407u) > (3090511079u)), vec4<bool>(true, (3997711594u) >= (3330791436u), all(vec4<bool>(true, false, false, true)), !(true)), !(select(vec4<bool>(false, false, true, true), vec4<bool>(true, true, false, true), vec4<bool>(true, true, false, false))))), (4085756870u) <= (453626101u), func_4(Struct_6(Struct_2(SAFE_MOD_u32(2955944581u, 3479335722u), Struct_1(vec2<i32>(var_0, -575072088)), Struct_1(vec2<i32>(var_0, var_0)), vec3<bool>(false, true, true), vec3<i32>(var_0, var_0, var_0), max(vec3<u32>(577582594u, 588687354u, 381440154u), vec3<u32>(2573732340u, 2083818132u, 4196634394u)), (var_0) >> (3780287699u), vec4<bool>(true, false, false, false)), Struct_4((89974991u) == (636703028u), Struct_3(998487273u, vec4<u32>(801497506u, 142322791u, 2456207964u, 3904422639u), vec2<i32>(var_0, -1288212557), Struct_1(vec2<i32>(var_0, var_0)), vec4<i32>(1346807192, 194479891, var_0, var_0), true, false, var_0, var_0, Struct_1(vec2<i32>(var_0, 597590136))), vec4<u32>(2439784632u, 1157139795u, 1262237845u, 2604113300u), ~(4025549788u)), vec3<u32>(SAFE_TIMES_u32(1331596514u, 267243474u), (3397358910u) & (2119672062u), 3122290992u), Struct_5(Struct_3(402341675u, vec4<u32>(3465751336u, 1785867466u, 2414399686u, 2049279675u), vec2<i32>(var_0, -440412601), Struct_1(vec2<i32>(1053435586, 1792327264)), vec4<i32>(1539521212, var_0, -472351090, 1560267831), false, false, var_0, var_0, Struct_1(vec2<i32>(-1232345570, var_0))), Struct_2(973163989u, Struct_1(vec2<i32>(411817153, -573631100)), Struct_1(vec2<i32>(-803980539, var_0)), vec3<bool>(false, false, true), vec3<i32>(682716954, 122189283, var_0), vec3<u32>(176683353u, 2287916699u, 3010777775u), -244905517, vec4<bool>(true, false, false, true)), !(vec4<bool>(false, true, true, false)), SAFE_PLUS_vec4_i32(vec4<i32>(-638107815, 973155403, 894496377, 1182090516), vec4<i32>(var_0, var_0, var_0, -2119878317)), 1982049834), max(~(2407305634u), clamp(1965585795u, 3736994107u, 4131444541u)), SAFE_MOD_u32(723473366u, 395763449u), (var_0) != (-(1729616190)), ~(~(vec4<u32>(1154332900u, 2598810002u, 168661649u, 917361660u))), Struct_3(3592874085u, abs(vec4<u32>(116102408u, 1992766577u, 2928171343u, 2936518602u)), SAFE_TIMES_vec2_i32(vec2<i32>(274195206, var_0), vec2<i32>(-829854467, -378366306)), Struct_1(vec2<i32>(var_0, var_0)), SAFE_MINUS_vec4_i32(vec4<i32>(var_0, -502216123, 456588822, var_0), vec4<i32>(var_0, var_0, -1537439913, 2097351219)), false, (true) & (false), SAFE_PLUS_i32(var_0, var_0), 586474122, Struct_1(vec2<i32>(var_0, -1479401865))), 495780614u)), var_0, Struct_1(abs(min(SAFE_DIVIDE_vec2_i32(vec2<i32>(var_0, var_0), vec2<i32>(1707745782, var_0)), -(vec2<i32>(1282131448, var_0))))));
    }
    return (clamp((-935328025) ^ ((var_0) << ((2586050464u) << (3437084604u))), var_0, -958088378)) >> (SAFE_MOD_u32((clamp(max(4058674172u, 25619057u), SAFE_PLUS_u32(4281798591u, 1061617675u), max(965240753u, 2054292277u))) & (~(~(3799194024u))), clamp(SAFE_TIMES_u32(~(3290266662u), ~(2323154867u)), ~(max(3045991632u, 3734100456u)), dot(SAFE_MOD_vec3_u32(vec3<u32>(3959114953u, 3662194204u, 1987084526u), vec3<u32>(967625115u, 2092192671u, 284767904u)), ~(vec3<u32>(1494723108u, 868417393u, 1091056473u))))));
}

fn func_2(arg_0: vec4<bool>, arg_1: u32, arg_2: bool) -> vec4<bool> {
    if (!((true) | (!(!((false) && (true)))))) {
        let var_0 = select(-(vec3<i32>(18271824, dot(vec3<i32>(612858739, 536942994, -798576817), -(vec3<i32>(-1037410693, -2078337960, 708663125))), dot(~(vec4<i32>(1145460257, 808153204, -1758211463, 1614802076)), ~(vec4<i32>(1449449648, 1874953162, -2090825899, 1542134684))))), clamp(select(SAFE_PLUS_vec3_i32(vec3<i32>(39091146, 1781677299, 642813560), ~(vec3<i32>(-1773051420, -1891667749, 1576648979))), ((vec3<i32>(-1219802308, -1777241391, 924531578)) ^ (vec3<i32>(-323500668, 500861584, 1899093137))) >> (vec3<u32>(1317340725u, 3267783801u, 395854947u)), vec3<bool>(all(vec3<bool>(true, false, true)), !(false), !(false))), vec3<i32>(SAFE_DIVIDE_i32(1873655983, dot(vec2<i32>(-877527273, -1152548682), vec2<i32>(-1289088879, -1034920826))), -1425509406, func_3(Struct_4(false, Struct_3(3947596636u, vec4<u32>(1755025217u, 3468110453u, 1402631547u, 2217856775u), vec2<i32>(1262468818, 1452091530), Struct_1(vec2<i32>(-977156784, -1740569308)), vec4<i32>(-806826318, -465549016, -1367595850, -74147505), true, false, 1355062071, -1169702613, Struct_1(vec2<i32>(367600287, 709463639))), vec4<u32>(1973761435u, 3823319192u, 2476131715u, 2443417536u), 499959468u), any(vec4<bool>(false, true, false, true)))), abs(vec3<i32>(2046869064, -(2010641294), min(292417530, 1229842051)))), !((false) | ((select(4037164692u, 291121916u, true)) < (2011508592u))));
    }
    let var_0 = Struct_1(min(~(vec2<i32>(~(615026522), 2032930117)), vec2<i32>(dot(abs(vec4<i32>(957228344, 762955686, -231601108, 1515010495)), ~(vec4<i32>(216635897, -362962193, 919537128, -633636034))), (2005938146) & (~(-1253128223)))));
    loop {
        if ((LOOP_COUNTERS[2u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[2u] = (LOOP_COUNTERS[2u]) + (1u);
        let var_1 = !((~((~(1060876898u)) << (706876894u))) >= (~(abs(dot(vec3<u32>(3214969625u, 4031689213u, 113928787u), vec3<u32>(1816525648u, 2441836408u, 2675474278u))))));
    }
    let var_1 = Struct_3(~(1187318292u), SAFE_MOD_vec4_u32((select(~(vec4<u32>(3199347470u, 309422853u, 4182073269u, 1856934257u)), min(vec4<u32>(3367269011u, 610375350u, 2472597742u, 1381908939u), vec4<u32>(2367250380u, 2603629900u, 4193658058u, 4212463144u)), vec4<bool>(true, false, false, false))) << (~(vec4<u32>(4003512187u, 2280585246u, 3310259435u, 322466601u))), ~(min(select(vec4<u32>(2519648704u, 1972918318u, 964269874u, 3262386309u), vec4<u32>(3374633080u, 1542844161u, 3208915316u, 4148101193u), false), SAFE_TIMES_vec4_u32(vec4<u32>(4097103104u, 4205126597u, 3793411249u, 633109298u), vec4<u32>(3428431302u, 133758762u, 1388013560u, 1084851942u))))), vec2<i32>(~(select(var_0.a.x, (347315277) ^ (-1968514489), false)), (-(dot(vec2<i32>(var_0.a.x, -1244443105), var_0.a))) ^ (var_0.a.x)), Struct_1(SAFE_MOD_vec2_i32(abs(var_0.a), max(SAFE_PLUS_vec2_i32(var_0.a, var_0.a), vec2<i32>(var_0.a.x, var_0.a.x)))), ~(SAFE_DIVIDE_vec4_i32(-(vec4<i32>(var_0.a.x, var_0.a.x, var_0.a.x, var_0.a.x)), -(select(vec4<i32>(var_0.a.x, 666253901, 1203027167, var_0.a.x), vec4<i32>(var_0.a.x, var_0.a.x, var_0.a.x, -709850746), false)))), !((false) | ((-1486015859) == (var_0.a.x))), (false) | (all(vec2<bool>(all(vec4<bool>(false, true, true, false)), any(vec3<bool>(false, false, false))))), (min(clamp(var_0.a.x, max(-501058, var_0.a.x), SAFE_DIVIDE_i32(var_0.a.x, var_0.a.x)), -(-475890758))) | (dot(SAFE_PLUS_vec4_i32(vec4<i32>(var_0.a.x, var_0.a.x, -1120957319, 937102797), clamp(vec4<i32>(var_0.a.x, var_0.a.x, var_0.a.x, var_0.a.x), vec4<i32>(-1464412312, 1554119104, 1907726735, 1882435467), vec4<i32>(var_0.a.x, -1591327047, -1123693352, var_0.a.x))), vec4<i32>(SAFE_PLUS_i32(507107665, var_0.a.x), ~(var_0.a.x), ~(var_0.a.x), SAFE_TIMES_i32(var_0.a.x, 430004001)))), ~(-(func_3(Struct_4(false, Struct_3(4048394033u, vec4<u32>(2376904663u, 3385966129u, 1822994754u, 2096116885u), vec2<i32>(var_0.a.x, 16407253), Struct_1(var_0.a), vec4<i32>(var_0.a.x, var_0.a.x, 2048323376, var_0.a.x), false, false, var_0.a.x, 63863428, Struct_1(var_0.a)), vec4<u32>(4226716091u, 3438155833u, 3046478546u, 815469680u), 4033914159u), false))), Struct_1(vec2<i32>(336944795, max((-1543657629) >> (51560773u), 1402090271))));
    var var_2 = true;
    return vec4<bool>(!(!(any(vec2<bool>(true, var_2)))), true, any(select(select(!(vec4<bool>(false, true, false, var_1.f)), !(vec4<bool>(var_2, false, false, false)), select(vec4<bool>(var_2, var_1.f, true, var_2), vec4<bool>(true, var_2, var_1.g, false), vec4<bool>(var_2, true, true, var_1.g))), !(vec4<bool>(var_2, var_1.f, true, false)), var_2)), (abs((1945152556) | (dot(vec2<i32>(var_1.c.x, var_0.a.x), var_0.a)))) < (-1993286343));
}

fn func_1() -> u32 {
    var var_0 = select(!(vec4<bool>(select(!(true), true, all(vec2<bool>(false, false))), false, true, !(!(true)))), select(select(select(select(vec4<bool>(false, false, false, true), vec4<bool>(false, false, true, true), true), !(vec4<bool>(false, false, true, false)), !(true)), func_2(!(vec4<bool>(false, true, true, true)), 1366895501u, (false) && (true)), !(!(vec4<bool>(false, false, true, true)))), vec4<bool>(all(select(vec3<bool>(false, true, false), vec3<bool>(false, true, false), vec3<bool>(true, false, false))), true, ((2443904564u) & (425403291u)) != (~(2378550040u)), false), all(vec2<bool>(!(false), !(true)))), (!(any(select(vec3<bool>(true, true, false), vec3<bool>(true, true, false), false)))) & (all(!(select(vec2<bool>(false, false), vec2<bool>(false, true), false)))));
    var_0.x = !(!(all(vec2<bool>(var_0.x, !(var_0.x)))));
    let var_1 = false;
    var_0 = var_0;
    var var_2 = Struct_4(any(select(vec2<bool>(!(true), select(var_1, var_1, var_0.x)), vec2<bool>(var_1, var_0.x), (true) && ((var_1) == (false)))), Struct_3(min(169806967u, 2651703832u), vec4<u32>(~(~(1938331144u)), ~(SAFE_MOD_u32(1192839769u, 1432588269u)), ~(~(3988051922u)), dot(~(vec2<u32>(1909974150u, 2585498872u)), (vec2<u32>(927069891u, 3810260646u)) | (vec2<u32>(2517637908u, 237964110u)))), SAFE_PLUS_vec2_i32(select(vec2<i32>(2009904671, -2082532001), ~(vec2<i32>(1539995580, 856828114)), var_0.yw), min(vec2<i32>(-85318909, -1839952355), ~(vec2<i32>(-614666280, 1894654126)))), Struct_1(~((vec2<i32>(-1923350505, 991899071)) << (vec2<u32>(2983946745u, 58797694u)))), select(-(vec4<i32>(2091347407, 578270346, 1194917055, -1389759362)), vec4<i32>(dot(vec4<i32>(1641216300, 1955898180, 1560062681, 1479565518), vec4<i32>(298488277, -1279472764, 261179751, 1016128495)), 1517215023, -1104891768, ~(1948529530)), false), var_0.x, !(var_0.x), -(dot(SAFE_DIVIDE_vec4_i32(vec4<i32>(-279851148, -1877304363, 1920883232, -2058130742), vec4<i32>(-1648816856, -92027388, -232245597, 260136580)), vec4<i32>(1271817667, 1557493310, 1838515410, -1312531865))), ~(1515753435), Struct_1(~(SAFE_PLUS_vec2_i32(vec2<i32>(357915446, -449347908), vec2<i32>(-1395644, -568707836))))), ~(vec4<u32>(671439000u, dot(select(vec3<u32>(776964316u, 2854617390u, 3158391897u), vec3<u32>(2247991110u, 515229109u, 598542219u), false), max(vec3<u32>(314003247u, 3564795699u, 2152758829u), vec3<u32>(3944783259u, 1003270067u, 2751671892u))), ~(dot(vec2<u32>(3414618222u, 3018341192u), vec2<u32>(1581412108u, 3702427283u))), 1737692051u)), 3610512269u);
    if (all(!(select(vec4<bool>(false, false, !(var_2.b.g), (var_0.x) || (false)), vec4<bool>(any(var_0.zy), var_1, all(vec2<bool>(var_2.b.g, var_0.x)), false), vec4<bool>(any(vec3<bool>(var_2.b.g, false, var_2.a)), !(false), var_2.b.f, !(var_0.x)))))) {
        var_0 = func_2(vec4<bool>(false, (68772474) >= (var_2.b.c.x), true, true), max(dot(~(clamp(var_2.b.b.www, var_2.b.b.wyx, vec3<u32>(var_2.b.b.x, var_2.b.b.x, var_2.d))), ~(~(var_2.b.b.zzy))), ~(~(var_2.b.b.x))), (func_4(Struct_6(Struct_2(var_2.c.x, Struct_1(var_2.b.c), Struct_1(var_2.b.e.xx), vec3<bool>(var_2.b.f, true, true), vec3<i32>(var_2.b.j.a.x, var_2.b.j.a.x, -1829645485), vec3<u32>(421362617u, 1966222081u, 29724620u), 1889660206, vec4<bool>(var_2.a, var_2.a, false, var_1)), Struct_4(true, var_2.b, var_2.b.b, 3436883760u), select(var_2.c.yyz, var_2.c.xxz, var_0.x), Struct_5(var_2.b, Struct_2(1043350160u, var_2.b.j, var_2.b.j, var_0.xxx, vec3<i32>(var_2.b.d.a.x, var_2.b.j.a.x, var_2.b.j.a.x), var_2.b.b.yyy, -1903020999, vec4<bool>(false, var_1, true, var_2.b.g)), var_0, var_2.b.e, var_2.b.h), abs(3990102753u), select(var_2.b.b.x, var_2.c.x, var_2.b.f), !(true), var_2.b.b, var_2.b, dot(var_2.b.b, vec4<u32>(654170537u, 704000408u, var_2.d, 2686054867u))))) > (-(-1803358077)));
    }
    var_0 = !(func_2(!(vec4<bool>((var_0.x) && (true), all(var_0.yxw), !(var_2.a), true)), (~(~(var_2.c.x))) >> (abs((2605558150u) & (1737955290u))), any(vec4<bool>(any(var_0), true, true, true))));
    let var_3 = select(any(vec2<bool>(true, true)), (var_2.b.h) >= (-(var_2.b.h)), (!(var_2.a)) && ((var_2.c.x) > (SAFE_MINUS_u32((var_2.b.a) | (var_2.c.x), max(3998767149u, var_2.b.b.x)))));
    return SAFE_MINUS_u32(var_2.c.x, 4140937214u);
}

@stage(compute)
@workgroup_size(1)
fn main() {
    if ((!(((func_1()) >> (~(3693228234u))) == (~(1258984485u)))) | (!(any(select(vec4<bool>(false, false, false, true), func_2(vec4<bool>(true, false, false, false), 2788853098u, true), !(false)))))) {
        var var_0 = ~(vec4<u32>((clamp(~(1134116342u), select(2863865374u, 3947583818u, false), ~(3107036149u))) >> (854023992u), 1526509829u, 1533108053u, SAFE_DIVIDE_u32(2150440906u, 3217233018u)));
    }
    if (all(select(!(vec4<bool>(any(vec4<bool>(true, false, true, false)), true, (false) | (true), !(true))), select(!(!(vec4<bool>(true, false, true, false))), vec4<bool>(true, any(vec3<bool>(false, true, true)), true, true), select(vec4<bool>(true, false, false, false), vec4<bool>(false, false, true, false), vec4<bool>(true, false, false, true))), !(select(func_2(vec4<bool>(true, false, false, true), 4057755834u, true), !(vec4<bool>(false, false, true, false)), !(true)))))) {
        let var_0 = Struct_1(min(-(~(SAFE_DIVIDE_vec2_i32(vec2<i32>(-1140380847, -1522686306), vec2<i32>(-1023234276, 1770084131)))), (~(clamp(vec2<i32>(1076671845, 888982104), vec2<i32>(-894887940, -393981846), vec2<i32>(-340130891, 433336462)))) ^ (SAFE_TIMES_vec2_i32(min(vec2<i32>(1922062649, 1301935199), vec2<i32>(223846830, 1134378852)), -(vec2<i32>(387354470, -1242025495))))));
    }
    let var_0 = 1978325215u;
    if (all(select(!(!(!(vec3<bool>(false, true, false)))), !(!(vec3<bool>(true, false, true))), all(select(vec2<bool>(true, false), select(vec2<bool>(false, true), vec2<bool>(false, false), false), (-1985057553) == (1755593697)))))) {
        if (any(vec2<bool>(any(!(select(vec2<bool>(true, false), vec2<bool>(true, false), false))), (~(var_0)) != (2048305959u)))) {
            var var_1 = all(vec3<bool>(any(select(select(vec4<bool>(false, false, true, true), vec4<bool>(false, false, true, false), false), !(vec4<bool>(false, false, false, true)), !(vec4<bool>(false, true, false, true)))), !(all(select(vec2<bool>(false, false), vec2<bool>(false, false), vec2<bool>(false, false)))), !(true)));
        }
    }
    if ((923905182) != (-(func_4(Struct_6(Struct_2(var_0, Struct_1(vec2<i32>(-892354546, 2082104879)), Struct_1(vec2<i32>(-1578060517, -942941654)), vec3<bool>(true, false, false), vec3<i32>(1969566793, 220510378, 1604270219), vec3<u32>(3003232978u, var_0, var_0), -1021932586, vec4<bool>(true, true, true, false)), Struct_4(false, Struct_3(var_0, vec4<u32>(var_0, 168415344u, 2679495839u, 3022450080u), vec2<i32>(538566800, -1452012043), Struct_1(vec2<i32>(-92605814, 560682245)), vec4<i32>(837590844, -1388365741, -447478911, -216384202), true, true, -1640866662, -474789634, Struct_1(vec2<i32>(-829651721, 826079934))), vec4<u32>(var_0, 2967652865u, 1425071030u, var_0), var_0), max(vec3<u32>(1249138122u, var_0, 552510144u), vec3<u32>(469653867u, var_0, var_0)), Struct_5(Struct_3(2526301529u, vec4<u32>(2224794790u, var_0, var_0, var_0), vec2<i32>(1129925429, -848459275), Struct_1(vec2<i32>(-1400484765, 1814204359)), vec4<i32>(-1925015968, 1848135024, 1112926251, 1324532947), true, true, 19428783, 125711449, Struct_1(vec2<i32>(409821742, 1413403274))), Struct_2(1967208834u, Struct_1(vec2<i32>(-83438405, -1194495788)), Struct_1(vec2<i32>(1665693768, -1364570644)), vec3<bool>(false, false, false), vec3<i32>(-2077770789, -811369231, 600500802), vec3<u32>(var_0, 2334332871u, var_0), 1607502010, vec4<bool>(false, false, false, false)), vec4<bool>(false, false, true, true), vec4<i32>(-324462001, 1554352274, 2020097448, 1427996359), 644474617), (3675194368u) & (93969352u), (1813725302u) | (1142587190u), !(false), SAFE_DIVIDE_vec4_u32(vec4<u32>(2736865587u, 3037429649u, var_0, var_0), vec4<u32>(var_0, 274011293u, var_0, 3883991367u)), Struct_3(var_0, vec4<u32>(865530416u, 894979877u, 2715241776u, 981277320u), vec2<i32>(-943600065, -163161375), Struct_1(vec2<i32>(-2116642726, -598511308)), vec4<i32>(-690939787, 283214754, 650977660, -114526481), true, false, -1666613186, 327841653, Struct_1(vec2<i32>(-381616811, 1722776466))), ~(642560227u)))))) {
        if (select(all(!(!(!(vec2<bool>(false, false))))), all(!(vec2<bool>(!(false), (false) && (false)))), true)) {
            loop {
                if ((LOOP_COUNTERS[0u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[0u] = (LOOP_COUNTERS[0u]) + (1u);
                let var_1 = Struct_3(SAFE_MOD_u32(4247493245u, var_0), vec4<u32>(min(abs(~(var_0)), 1494518971u), SAFE_PLUS_u32(select(1221323455u, var_0, false), var_0), 2672507304u, ~(select(var_0, max(var_0, var_0), all(vec2<bool>(false, false))))), vec2<i32>(-2020740981, max(dot(vec3<i32>(45604315, -1985499413, 127987281), SAFE_PLUS_vec3_i32(vec3<i32>(-100815873, -1101845420, 533620404), vec3<i32>(-1235481197, -1595480156, 1817025920))), clamp(~(1576815868), -(-996054281), func_3(Struct_4(true, Struct_3(1056030412u, vec4<u32>(var_0, var_0, 2053963984u, var_0), vec2<i32>(-1709974330, -1462651652), Struct_1(vec2<i32>(-1975633237, 1496580415)), vec4<i32>(-1754322631, -902450861, -1622834473, -937757044), false, false, -1350711266, 1134784014, Struct_1(vec2<i32>(21896557, 1249516757))), vec4<u32>(392952036u, var_0, var_0, 352956942u), var_0), true)))), Struct_1(select((SAFE_MINUS_vec2_i32(vec2<i32>(-1021255688, 1825022750), vec2<i32>(1742055272, -1583179784))) ^ (abs(vec2<i32>(1443076703, -2145101709))), SAFE_MOD_vec2_i32(vec2<i32>(-1604017201, -1243689070), vec2<i32>(-507654026, 1964637104)), !(all(vec3<bool>(false, false, false))))), vec4<i32>(-(SAFE_DIVIDE_i32(40906238, dot(vec4<i32>(1833231511, 79041414, -613729242, -1779644830), vec4<i32>(936474802, -874699796, 1352438682, -1131912543)))), ~(-1802828972), ~(~(dot(vec3<i32>(-1547619669, -1958999124, 1542938081), vec3<i32>(1464818395, 173069552, -742898643)))), abs(-(~(-882535672)))), all(!(select(!(vec2<bool>(false, false)), !(vec2<bool>(false, true)), vec2<bool>(true, true)))), !((~(2200550342u)) > (SAFE_TIMES_u32(~(968660458u), 1422882889u))), ~(1882278549), -(-1244516197), Struct_1((-(~(vec2<i32>(827926932, 1987917576)))) << (clamp(vec2<u32>(1104982635u, var_0), vec2<u32>(var_0, var_0), ~(vec2<u32>(1692867729u, 1719779291u))))));
            }
        }
    }
    if (((!(!((true) & (true)))) & (false)) || (true)) {
        if (false) {
            var var_1 = (~(vec3<u32>(~(SAFE_MOD_u32(502304304u, 954949415u)), SAFE_TIMES_u32((var_0) << (var_0), ~(388185657u)), SAFE_MOD_u32(~(2560872785u), 3863873213u)))) >> ((vec3<u32>(~(~(4214579854u)), ~(~(var_0)), var_0)) & (max(SAFE_DIVIDE_vec3_u32(SAFE_MOD_vec3_u32(vec3<u32>(271415142u, 2262266016u, 3676244873u), vec3<u32>(var_0, 26756228u, var_0)), select(vec3<u32>(3969324996u, var_0, var_0), vec3<u32>(2719429712u, var_0, 1369528383u), true)), vec3<u32>(var_0, (2316822878u) >> (1071526739u), ~(var_0)))));
        }
    }
    output.value = input.value;
    output.value = dot(~(select(vec4<u32>(106921564u, 2832646104u, (var_0) | (2224257317u), (3939493151u) << (3222880078u)), vec4<u32>(dot(vec4<u32>(var_0, var_0, 4138656856u, 2242458904u), vec4<u32>(197151255u, var_0, var_0, var_0)), var_0, 304850097u, ~(2053782297u)), vec4<bool>(false, !(false), !(true), false))), vec4<u32>((102830761u) << (278461741u), var_0, 4208153264u, (dot(SAFE_DIVIDE_vec2_u32(vec2<u32>(2126397999u, 3293968286u), vec2<u32>(var_0, 2384170465u)), vec2<u32>(3971033353u, 1867700196u))) >> (~(SAFE_DIVIDE_u32(var_0, var_0)))));
}

