// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[3,204,16,37]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}
// Seed: 8943013669557091515

var<private> LOOP_COUNTERS: array<u32, 10>;

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
    a: vec2<u32>;
    b: u32;
    c: vec2<u32>;
    d: u32;
    e: u32;
    f: vec3<i32>;
    g: i32;
};

struct Struct_2 {
    a: bool;
    b: vec2<i32>;
    c: vec2<bool>;
    d: vec4<bool>;
};

struct Struct_3 {
    a: bool;
    b: i32;
    c: Struct_1;
    d: i32;
    e: i32;
    f: Struct_1;
    g: Struct_2;
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

fn SAFE_PLUS_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y));
}

fn SAFE_PLUS_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y), SAFE_PLUS_u32(a.z, b.z), SAFE_PLUS_u32(a.w, b.w));
}

fn SAFE_MINUS_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_MINUS_i32(a.x, b.x), SAFE_MINUS_i32(a.y, b.y), SAFE_MINUS_i32(a.z, b.z), SAFE_MINUS_i32(a.w, b.w));
}

fn SAFE_MINUS_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y));
}

fn SAFE_MINUS_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_MINUS_u32(a.x, b.x), SAFE_MINUS_u32(a.y, b.y), SAFE_MINUS_u32(a.z, b.z), SAFE_MINUS_u32(a.w, b.w));
}

fn SAFE_TIMES_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y), SAFE_TIMES_i32(a.z, b.z));
}

fn SAFE_TIMES_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y), SAFE_TIMES_i32(a.z, b.z), SAFE_TIMES_i32(a.w, b.w));
}

fn SAFE_TIMES_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y));
}

fn SAFE_TIMES_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_TIMES_u32(a.x, b.x), SAFE_TIMES_u32(a.y, b.y), SAFE_TIMES_u32(a.z, b.z), SAFE_TIMES_u32(a.w, b.w));
}

fn SAFE_DIVIDE_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y), SAFE_DIVIDE_i32(a.z, b.z));
}

fn SAFE_DIVIDE_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y));
}

fn SAFE_DIVIDE_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_DIVIDE_u32(a.x, b.x), SAFE_DIVIDE_u32(a.y, b.y), SAFE_DIVIDE_u32(a.z, b.z));
}

fn SAFE_MOD_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_MOD_i32(a.x, b.x), SAFE_MOD_i32(a.y, b.y));
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

fn SAFE_MOD_vec3_u32(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
    return vec3<u32>(SAFE_MOD_u32(a.x, b.x), SAFE_MOD_u32(a.y, b.y), SAFE_MOD_u32(a.z, b.z));
}

fn func_4(arg_0: Struct_1) -> u32 {
    let var_0 = Struct_1(vec2<u32>((clamp(~(222758995u), 421882145u, abs(1663631763u))) & ((SAFE_TIMES_u32(3955757622u, 1685569963u)) << (3533241655u)), ~(1764263188u)), clamp((dot(SAFE_MOD_vec3_u32(vec3<u32>(372096854u, 123915337u, 166802995u), vec3<u32>(3998936721u, 146068411u, 420503086u)), clamp(vec3<u32>(680513585u, 3730221377u, 2459669082u), vec3<u32>(2108034833u, 3069984617u, 3500119688u), vec3<u32>(1954818840u, 3181074226u, 3593289046u)))) << (~(SAFE_MINUS_u32(169422087u, 3395713450u))), ~(1034652241u), 3795854857u), ((select(~(vec2<u32>(299639176u, 725394539u)), vec2<u32>(277709728u, 1705754339u), !(vec2<bool>(true, true)))) >> (max(vec2<u32>(929468053u, 3927525698u), vec2<u32>(2924000045u, 4244753252u)))) ^ ((clamp((vec2<u32>(4153547514u, 3077315330u)) & (vec2<u32>(509200056u, 99012908u)), vec2<u32>(4043906152u, 4275645442u), select(vec2<u32>(244253442u, 1884611024u), vec2<u32>(524010982u, 2641203761u), vec2<bool>(true, true)))) >> ((vec2<u32>(3947015967u, 2217811756u)) | (SAFE_DIVIDE_vec2_u32(vec2<u32>(3784748582u, 2161439249u), vec2<u32>(437253683u, 3383158895u))))), 2562494166u, ~(dot(SAFE_DIVIDE_vec3_u32(~(vec3<u32>(608468527u, 3684883315u, 431178084u)), max(vec3<u32>(3993096960u, 108652030u, 2406350474u), vec3<u32>(2246311439u, 1720676459u, 2976127679u))), max((vec3<u32>(1040521578u, 4052853375u, 906125004u)) & (vec3<u32>(2300878519u, 3898002297u, 2301625891u)), vec3<u32>(4227290706u, 4025463303u, 3347319826u)))), vec3<i32>(-(dot(-(vec3<i32>(-1213063332, 915224898, 1465188816)), -(vec3<i32>(-1512527458, 1307050498, 1937159062)))), 226067433, (SAFE_MINUS_i32(-618051742, -195591423)) & (-665172320)), (-(dot(max(vec2<i32>(1517857218, -1589273632), vec2<i32>(-1350063831, 859289257)), min(vec2<i32>(1911543753, -262125266), vec2<i32>(1324339887, 1010164144))))) >> (~(~(~(1361454234u)))));
    loop {
        if ((LOOP_COUNTERS[1u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[1u] = (LOOP_COUNTERS[1u]) + (1u);
        let var_1 = var_0;
    }
    if (true) {
        if ((true) || ((!(any(vec2<bool>(false, false)))) && (true))) {
            loop {
                if ((LOOP_COUNTERS[2u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[2u] = (LOOP_COUNTERS[2u]) + (1u);
                if (all(!(select(vec3<bool>(!(false), !(false), (true) | (false)), vec3<bool>(false, false, !(false)), !((false) | (false)))))) {
                    if (!(true)) {
                        let var_1 = Struct_3(false, var_0.f.x, var_0, -(~(~((1179186973) >> (4150127394u)))), (SAFE_MOD_i32(var_0.g, dot(vec3<i32>(887600908, 762802517, 867963665), vec3<i32>(1743554606, var_0.g, -440271342)))) << (~((76891853u) >> (~(1157091236u)))), var_0, Struct_2((-680101048) > (~(~(1722078303))), ~(~(var_0.f.zz)), !(select(vec2<bool>(false, true), !(vec2<bool>(true, false)), vec2<bool>(false, false))), !(select(vec4<bool>(true, true, true, true), !(vec4<bool>(false, false, false, false)), select(vec4<bool>(false, false, false, false), vec4<bool>(true, false, true, true), false)))));
                    }
                }
            }
        }
    }
    var var_1 = dot(max(vec4<i32>((dot(vec4<i32>(-34982926, var_0.g, 152775064, var_0.f.x), vec4<i32>(var_0.f.x, var_0.f.x, 1515622859, var_0.f.x))) & (min(var_0.g, var_0.g)), 1934348356, var_0.f.x, 818235796), select(min(vec4<i32>(var_0.g, var_0.g, 1717819771, 1756619715), vec4<i32>(1611248674, var_0.g, var_0.g, -2061715539)), SAFE_MOD_vec4_i32(min(vec4<i32>(var_0.f.x, var_0.g, var_0.g, 456731370), vec4<i32>(1391296074, var_0.f.x, 979023155, var_0.g)), (vec4<i32>(var_0.g, -924271170, -1865481010, var_0.g)) | (vec4<i32>(1769256416, var_0.g, -88547531, var_0.g))), false)), abs((~((vec4<i32>(var_0.g, -690498420, -1537511961, -627997879)) & (vec4<i32>(-1492668124, -912685494, var_0.g, -432377948)))) << ((max(vec4<u32>(var_0.c.x, 2327610978u, var_0.e, var_0.b), vec4<u32>(1853814288u, var_0.d, var_0.b, var_0.c.x))) << (SAFE_MINUS_vec4_u32(vec4<u32>(var_0.e, var_0.b, var_0.c.x, var_0.c.x), vec4<u32>(var_0.e, 3184351043u, 130669688u, 1507909666u))))));
    let var_2 = !(!(!(!(select(vec2<bool>(false, true), vec2<bool>(false, false), vec2<bool>(false, true))))));
    return 2869957783u;
}

fn func_3(arg_0: Struct_1) -> Struct_2 {
    let var_0 = vec4<u32>(~(614400864u), 2334956404u, abs(dot(vec4<u32>(max(2466057961u, 4212138898u), (1832135861u) << (2018671982u), 2889153860u, dot(vec2<u32>(311090834u, 1322994913u), vec2<u32>(2328242004u, 3376479142u))), ~(~(vec4<u32>(3261387077u, 2657008294u, 3796674002u, 4181865192u))))), min(~(3529633428u), 156903060u));
    if (false) {
        let var_1 = !(true);
    }
    let var_1 = clamp(~(1737130674), select(~(885481218), -(760207805), !(all(!(vec4<bool>(true, true, true, true))))), -169434578);
    return Struct_2(true, vec2<i32>(var_1, ~(539842754)), vec2<bool>(!(any(vec4<bool>(true, true, true, true))), !(select(all(vec3<bool>(false, true, false)), (false) & (false), any(vec4<bool>(true, false, false, false))))), !(vec4<bool>((!(true)) & (true), all(select(vec4<bool>(true, true, true, false), vec4<bool>(false, true, true, false), false)), (2242234073u) == (SAFE_TIMES_u32(var_0.x, var_0.x)), (any(vec2<bool>(false, false))) & (any(vec4<bool>(true, false, false, false))))));
}

fn func_2(arg_0: Struct_1, arg_1: Struct_3, arg_2: vec4<u32>) -> u32 {
    let var_0 = func_3(Struct_1(vec2<u32>(1712739143u, SAFE_MOD_u32(4027793211u, 2342787325u)), abs(391460274u), ((vec2<u32>(1601067377u, 4107002920u)) << ((vec2<u32>(2406167896u, 3829708755u)) & (vec2<u32>(2297426528u, 363440530u)))) << (vec2<u32>((4204458228u) << (582385650u), ~(915350801u))), 111508718u, abs(3949008609u), abs(-(vec3<i32>(1908745992, -528411309, 333999592))), 455459969));
    loop {
        if ((LOOP_COUNTERS[3u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[3u] = (LOOP_COUNTERS[3u]) + (1u);
        let var_1 = ~(~(SAFE_TIMES_vec2_u32(~((vec2<u32>(176626999u, 3017168087u)) & (vec2<u32>(3579520102u, 3525298992u))), max(vec2<u32>(2682031012u, 3785764138u), ~(vec2<u32>(1513327721u, 4160817146u))))));
    }
    if ((!(any(!(var_0.d)))) & (!((-1212628042) != ((var_0.b.x) & (SAFE_MOD_i32(-419320581, var_0.b.x)))))) {
        if (var_0.a) {
            if ((SAFE_MINUS_u32(1481203620u, ~(2086292702u))) > (~((1062788347u) ^ (SAFE_MINUS_u32(~(1584664261u), abs(821313319u)))))) {
                {
                    loop {
                        if ((LOOP_COUNTERS[4u]) >= (1u)) {
                            break;
                        }
                        LOOP_COUNTERS[4u] = (LOOP_COUNTERS[4u]) + (1u);
                        var var_1 = SAFE_TIMES_u32(SAFE_MOD_u32(3302717618u, SAFE_MINUS_u32(SAFE_TIMES_u32(2714993299u, 592165337u), func_4(Struct_1(vec2<u32>(1047942126u, 2242578849u), 2892413267u, vec2<u32>(1749407639u, 2027599030u), 1894841468u, 3636142055u, vec3<i32>(-302821537, var_0.b.x, var_0.b.x), 955525734)))), dot(~(~(~(vec3<u32>(502156509u, 3155949781u, 3540936987u)))), vec3<u32>(dot((vec2<u32>(2901217043u, 2507271948u)) & (vec2<u32>(2737622782u, 191047377u)), SAFE_PLUS_vec2_u32(vec2<u32>(601046019u, 1667107398u), vec2<u32>(319474312u, 2782820014u))), (2051724234u) ^ (abs(3206441850u)), select(2263429117u, 927996142u, !(true)))));
                    }
                }
            }
        }
    }
    if (all(!(var_0.d.wyx))) {
        if (var_0.d.x) {
            let var_1 = Struct_3(var_0.c.x, (-846055500) | (-814615262), Struct_1(SAFE_PLUS_vec2_u32(vec2<u32>(SAFE_DIVIDE_u32(944140744u, 3055243982u), ~(1638420887u)), vec2<u32>(2713025872u, (1647823106u) | (4130937654u))), 404936525u, vec2<u32>(992488314u, (3958704575u) << (116918779u)), func_4(Struct_1(clamp(vec2<u32>(2102874673u, 3017385434u), vec2<u32>(2319300801u, 3672931634u), vec2<u32>(1381356563u, 2346449413u)), ~(1492363229u), vec2<u32>(454134454u, 3142562933u), ~(980481910u), 445736237u, abs(vec3<i32>(var_0.b.x, -2120857429, var_0.b.x)), 1206253416)), ~(3329105504u), -(~(abs(vec3<i32>(var_0.b.x, 2050178877, -306316909)))), var_0.b.x), ~(~(var_0.b.x)), -(dot(select(-(vec3<i32>(var_0.b.x, var_0.b.x, var_0.b.x)), SAFE_PLUS_vec3_i32(vec3<i32>(var_0.b.x, var_0.b.x, 1394973958), vec3<i32>(var_0.b.x, var_0.b.x, var_0.b.x)), !(var_0.c.x)), select((vec3<i32>(var_0.b.x, var_0.b.x, 190863936)) | (vec3<i32>(-1085892278, var_0.b.x, var_0.b.x)), max(vec3<i32>(-1088218530, var_0.b.x, -1843484570), vec3<i32>(-660778340, var_0.b.x, -1967167129)), false))), Struct_1((vec2<u32>(950943885u, 2754526497u)) & (SAFE_MINUS_vec2_u32(vec2<u32>(2392151980u, 4228097159u), min(vec2<u32>(1196454182u, 1084194390u), vec2<u32>(3069882568u, 1237979327u)))), min(~(2585234541u), ~(select(2343793173u, 3030828488u, var_0.d.x))), ~((~(vec2<u32>(2253398689u, 3001079617u))) | (~(vec2<u32>(3755253995u, 1168760359u)))), ~(~((1820938934u) | (426884191u))), (~(3203003806u)) >> (SAFE_PLUS_u32(~(3801112446u), 571987589u)), (vec3<i32>(var_0.b.x, dot(vec4<i32>(910046180, var_0.b.x, var_0.b.x, 91936375), vec4<i32>(var_0.b.x, 2141612380, var_0.b.x, var_0.b.x)), clamp(var_0.b.x, -677175280, 1730437371))) | (-(-(vec3<i32>(1422116160, var_0.b.x, var_0.b.x)))), dot(-(abs(vec2<i32>(var_0.b.x, var_0.b.x))), min(vec2<i32>(var_0.b.x, -934780967), var_0.b))), func_3(Struct_1((~(vec2<u32>(3812173991u, 3675448377u))) >> (~(vec2<u32>(719027996u, 877205050u))), ~(4121108264u), ~(vec2<u32>(1821116096u, 2810854532u)), (~(1695374872u)) ^ (~(2328744332u)), (max(854225641u, 2702922443u)) ^ (1426026892u), -(SAFE_DIVIDE_vec3_i32(vec3<i32>(var_0.b.x, var_0.b.x, var_0.b.x), vec3<i32>(1422607143, var_0.b.x, var_0.b.x))), -(dot(vec3<i32>(-98830435, -2050377968, -1554941525), vec3<i32>(var_0.b.x, -1033873858, -444211700))))));
        }
    }
    if (all(select(select(!(select(var_0.d.wyw, vec3<bool>(var_0.a, var_0.c.x, true), var_0.d.x)), var_0.d.xzx, var_0.d.x), select(var_0.d.zzz, var_0.d.yxz, var_0.d.wxz), select(var_0.d.xxy, var_0.d.xww, false)))) {
        let var_1 = !((var_0.c.x) & (any(var_0.d)));
    }
    loop {
        if ((LOOP_COUNTERS[5u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[5u] = (LOOP_COUNTERS[5u]) + (1u);
        var var_1 = clamp(1874050984u, (dot(abs(vec4<u32>(3733533193u, 3639962665u, 155150832u, 3528289017u)), ~(vec4<u32>(3740787815u, 1816897097u, 3217008353u, 2124819166u)))) >> (~(dot((vec3<u32>(1387956866u, 1719068934u, 3173260598u)) ^ (vec3<u32>(2862237304u, 953028611u, 375991990u)), ~(vec3<u32>(1468341811u, 958201819u, 40616886u))))), max(dot(clamp(~(vec3<u32>(3183902933u, 3051898572u, 1695675349u)), vec3<u32>(3219828112u, 2594815993u, 3304493155u), ~(vec3<u32>(412373452u, 3237967372u, 1382808155u))), select((vec3<u32>(2581298849u, 13286852u, 180070520u)) | (vec3<u32>(1201044023u, 4227296577u, 2270505411u)), ~(vec3<u32>(1297081719u, 2283312935u, 3496843318u)), vec3<bool>(false, false, false))), 285314812u));
    }
    loop {
        if ((LOOP_COUNTERS[6u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[6u] = (LOOP_COUNTERS[6u]) + (1u);
        loop {
            if ((LOOP_COUNTERS[7u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[7u] = (LOOP_COUNTERS[7u]) + (1u);
            let var_1 = 348192363;
        }
    }
    let var_1 = Struct_1(~((clamp(SAFE_MINUS_vec2_u32(vec2<u32>(4247852272u, 2244606527u), vec2<u32>(1126911291u, 352241579u)), vec2<u32>(3596039098u, 4098169017u), select(vec2<u32>(1643110554u, 684144225u), vec2<u32>(1986479033u, 928744810u), var_0.a))) >> (max(abs(vec2<u32>(313172890u, 663429030u)), ~(vec2<u32>(1788034149u, 1584932315u))))), (~(1880643030u)) | (~(4070265827u)), SAFE_PLUS_vec2_u32((~(vec2<u32>(3518722464u, 2625820903u))) >> (vec2<u32>((3597366225u) ^ (3903915374u), select(1429636719u, 4275555763u, var_0.c.x))), select(~((vec2<u32>(682470834u, 2157256330u)) & (vec2<u32>(2192857961u, 4119966969u))), vec2<u32>(357281731u, 3184194300u), var_0.c)), SAFE_DIVIDE_u32((195850141u) & ((~(1305952103u)) ^ (1994410469u)), 3968568222u), ~(SAFE_PLUS_u32(1537187572u, ~(~(923417476u)))), min(vec3<i32>(dot(~(vec2<i32>(699424928, var_0.b.x)), ~(var_0.b)), SAFE_MOD_i32(var_0.b.x, ~(var_0.b.x)), 664538807), vec3<i32>(-(var_0.b.x), 1578034238, -1715914200)), ~(-159330873));
    let var_2 = var_1;
    loop {
        if ((LOOP_COUNTERS[8u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[8u] = (LOOP_COUNTERS[8u]) + (1u);
        var var_3 = ~(2774502024u);
    }
    return 2846095265u;
}

fn func_1(arg_0: bool, arg_1: bool, arg_2: vec2<u32>, arg_3: Struct_3) -> bool {
    {
        var var_0 = dot(abs(vec2<u32>(2882152011u, ~(4014368270u))), vec2<u32>(((2376401610u) | (~(93191667u))) ^ ((SAFE_TIMES_u32(772786073u, 876717330u)) & ((1633714453u) & (2264187821u))), (func_2(Struct_1(vec2<u32>(1788107226u, 712445057u), 1548476911u, vec2<u32>(3545479770u, 1428943280u), 3876787574u, 394600677u, vec3<i32>(2072127828, 1870988384, -136909034), 1233935003), Struct_3(true, -1420572911, Struct_1(vec2<u32>(3806802584u, 90439744u), 1555195110u, vec2<u32>(2969265328u, 274912398u), 2621899477u, 2493602524u, vec3<i32>(-1185237890, 1883994523, -741432249), -1899724892), -682131972, 79073675, Struct_1(vec2<u32>(1892460622u, 868363074u), 3553046991u, vec2<u32>(2581461017u, 592764103u), 2972934097u, 824763502u, vec3<i32>(-165438183, -641358138, -1739021474), -702985700), Struct_2(false, vec2<i32>(-49499128, -949745342), vec2<bool>(true, false), vec4<bool>(false, true, true, true))), (vec4<u32>(1182294882u, 946828273u, 121432960u, 1196736852u)) & (vec4<u32>(3441860177u, 1943022358u, 1112695064u, 991690418u)))) << (clamp(~(1229119489u), func_4(Struct_1(vec2<u32>(823818807u, 244961473u), 3912177086u, vec2<u32>(2440463558u, 2430523817u), 2366220905u, 1167264706u, vec3<i32>(-1296866892, -428938894, 350388896), -478177325)), ~(874097580u)))));
    }
    let var_0 = Struct_2(all(select(!(select(vec2<bool>(false, false), vec2<bool>(false, false), true)), !(vec2<bool>(true, false)), false)), vec2<i32>(max(1838921722, abs(~(952023567))), SAFE_MOD_i32(SAFE_DIVIDE_i32(-633134780, ~(-438271270)), max(SAFE_MOD_i32(-327887138, 880183718), 1749728436))), vec2<bool>((dot(select(vec2<u32>(389679295u, 2345107792u), vec2<u32>(1091875264u, 3555533174u), vec2<bool>(false, false)), ~(vec2<u32>(2625991813u, 3421154145u)))) > (func_2(Struct_1(vec2<u32>(3422553634u, 1900216633u), 4025372272u, vec2<u32>(2377686647u, 4009948871u), 1496292710u, 2095085476u, vec3<i32>(-2125792463, 839537615, 518678323), -348627304), Struct_3(true, -942630280, Struct_1(vec2<u32>(1076846759u, 1508001669u), 4140667175u, vec2<u32>(2122047644u, 3821516054u), 4155040110u, 3207759134u, vec3<i32>(-1458545797, 736628510, 255700057), -1902000556), 640345837, 94368562, Struct_1(vec2<u32>(2019079915u, 4129973192u), 1275332168u, vec2<u32>(256767820u, 3251564451u), 1714891549u, 1263432339u, vec3<i32>(-1446434448, 222039714, 265779376), 54841856), Struct_2(true, vec2<i32>(-1724731729, -230729827), vec2<bool>(true, false), vec4<bool>(false, false, true, true))), vec4<u32>(3709531966u, 1845134704u, 603306530u, 3822924172u))), (313635263) != (2091308308)), vec4<bool>(all(vec3<bool>(false, all(vec2<bool>(true, false)), false)), false, !(false), true));
    if (false) {
        var var_1 = (vec2<u32>(~(dot(select(vec4<u32>(2862378644u, 1308703803u, 678077913u, 3797779600u), vec4<u32>(1516621397u, 645549450u, 3536880969u, 2503958415u), vec4<bool>(var_0.a, var_0.c.x, var_0.c.x, var_0.d.x)), SAFE_PLUS_vec4_u32(vec4<u32>(3427361559u, 4127359586u, 2739695170u, 267258136u), vec4<u32>(119810899u, 3822879097u, 1208156276u, 1689314892u)))), clamp(~(SAFE_DIVIDE_u32(3518793577u, 1482249026u)), SAFE_MINUS_u32(3361257630u, ~(2914133175u)), 950360420u))) & (vec2<u32>(abs(dot(SAFE_TIMES_vec4_u32(vec4<u32>(3183419269u, 2572186550u, 3479749536u, 1478213871u), vec4<u32>(609338257u, 814559408u, 3658242609u, 878238456u)), select(vec4<u32>(1554319065u, 2473821408u, 1513011188u, 3132247522u), vec4<u32>(1139981211u, 3429050648u, 852349581u, 1626484444u), var_0.c.x))), ~(dot(max(vec4<u32>(28089798u, 346739716u, 3402980398u, 1847473919u), vec4<u32>(38711353u, 1485100546u, 4027409567u, 3410789343u)), min(vec4<u32>(883283945u, 1278589032u, 2025046748u, 2193161512u), vec4<u32>(919738336u, 810766245u, 1733795095u, 4198209522u))))));
    }
    if ((~(abs(-(1734587568)))) > (597147743)) {
        let var_1 = func_3(Struct_1((vec2<u32>(SAFE_MOD_u32(3190831280u, 1757480635u), (3740635780u) & (1408589932u))) >> ((vec2<u32>(1570770303u, 1363263814u)) | (select(vec2<u32>(1347632379u, 3868095710u), vec2<u32>(1938019711u, 803184900u), var_0.c))), (select(SAFE_MOD_u32(700407166u, 4029637793u), 978820725u, (var_0.a) & (var_0.d.x))) | ((SAFE_PLUS_u32(4228135414u, 983289930u)) << (dot(vec3<u32>(3840331727u, 3564603264u, 3465049573u), vec3<u32>(3133095974u, 2745307666u, 2226554324u)))), ~(~(~(vec2<u32>(1519579515u, 3088030155u)))), dot((vec2<u32>(2729852447u, 3391203069u)) & (~(vec2<u32>(2275649590u, 3349355292u))), select(vec2<u32>(2629789681u, 3142755346u), select(vec2<u32>(2738815380u, 621624621u), vec2<u32>(2845303365u, 4060068797u), false), false)), func_4(Struct_1(vec2<u32>(3180744345u, 412005952u), 4283471210u, vec2<u32>(3836058648u, 3007157363u), SAFE_MINUS_u32(2809645417u, 1841961805u), dot(vec3<u32>(3555266968u, 1858255223u, 3115685180u), vec3<u32>(788936914u, 977935418u, 114540825u)), vec3<i32>(1835243412, var_0.b.x, var_0.b.x), ~(var_0.b.x))), (select(vec3<i32>(-445554209, -2095058764, var_0.b.x), ~(vec3<i32>(var_0.b.x, -1097344059, -1794797133)), !(var_0.d.zzw))) & (~(max(vec3<i32>(var_0.b.x, -1132380434, var_0.b.x), vec3<i32>(-1739908028, var_0.b.x, var_0.b.x)))), ~(var_0.b.x)));
    }
    if (any(!(!(var_0.d)))) {
        var var_1 = Struct_3(any(!(select(!(var_0.d.xzy), select(vec3<bool>(false, true, var_0.d.x), var_0.d.xxw, var_0.a), select(var_0.d.wzx, vec3<bool>(var_0.d.x, true, var_0.a), true)))), -172191948, Struct_1(~(select(clamp(vec2<u32>(1423084773u, 1912719721u), vec2<u32>(695331399u, 4138868823u), vec2<u32>(496436781u, 3762322395u)), select(vec2<u32>(264325047u, 850239891u), vec2<u32>(883001365u, 2801601121u), true), (true) | (false))), ~(SAFE_TIMES_u32(clamp(1550367955u, 215799652u, 317847714u), ~(2207390363u))), SAFE_TIMES_vec2_u32(select(~(vec2<u32>(2366753976u, 2403343586u)), max(vec2<u32>(397527508u, 3377037923u), vec2<u32>(1971788158u, 3039441835u)), vec2<bool>(true, var_0.c.x)), ~((vec2<u32>(3398685523u, 869769468u)) >> (vec2<u32>(166629806u, 2404491247u)))), dot(select(vec3<u32>(598089367u, 2707983619u, 2529663599u), vec3<u32>(2689784003u, 3685748181u, 3084393843u), vec3<bool>(false, var_0.c.x, var_0.a)), ~(min(vec3<u32>(520318209u, 42512771u, 3712418941u), vec3<u32>(3386634962u, 3548561963u, 3255329311u)))), clamp(SAFE_TIMES_u32(~(533241888u), ~(327376972u)), SAFE_MOD_u32((855746199u) << (110354067u), ~(326237669u)), 2382027595u), vec3<i32>(~(min(var_0.b.x, var_0.b.x)), var_0.b.x, clamp((-74694799) >> (2040961053u), -1517846090, SAFE_MINUS_i32(-1480487034, -1414034977))), -927111679), -((var_0.b.x) & ((-663816945) ^ (dot(vec4<i32>(-2015390786, var_0.b.x, -1897028973, var_0.b.x), vec4<i32>(-1534760354, var_0.b.x, 1358334970, var_0.b.x))))), var_0.b.x, Struct_1(~(select(select(vec2<u32>(3755499316u, 3391897445u), vec2<u32>(1598570457u, 667293658u), false), vec2<u32>(1955263939u, 3085841467u), vec2<bool>(true, true))), dot(abs((vec3<u32>(536253017u, 2007573078u, 3985267448u)) | (vec3<u32>(2707657363u, 2799502614u, 595871756u))), ~(max(vec3<u32>(4200487528u, 1134537794u, 3055099022u), vec3<u32>(4076685766u, 3422951873u, 3670333892u)))), SAFE_MOD_vec2_u32((~(vec2<u32>(2539331398u, 1123869088u))) ^ (select(vec2<u32>(4270541885u, 3779023479u), vec2<u32>(3264454508u, 1883052267u), var_0.a)), (abs(vec2<u32>(95733882u, 920457201u))) >> (~(vec2<u32>(3923147979u, 3117788441u)))), (2193856381u) << (~(~(2614614461u))), select(dot(vec2<u32>(732021345u, 1809163413u), vec2<u32>(745937214u, 2098980090u)), (1913628608u) | (2780654010u), select(all(vec3<bool>(false, false, true)), true, !(var_0.c.x))), (-((vec3<i32>(1702878209, -1996985327, 300337882)) & (vec3<i32>(var_0.b.x, 1947405024, var_0.b.x)))) & (vec3<i32>(abs(89977208), ~(var_0.b.x), min(var_0.b.x, var_0.b.x))), (dot(SAFE_MOD_vec2_i32(vec2<i32>(var_0.b.x, var_0.b.x), vec2<i32>(-1340988151, 1144625918)), SAFE_PLUS_vec2_i32(var_0.b, vec2<i32>(var_0.b.x, var_0.b.x)))) >> (SAFE_PLUS_u32(~(1954718118u), func_2(Struct_1(vec2<u32>(2159222674u, 1308663426u), 959637272u, vec2<u32>(3356151316u, 847461863u), 1491263325u, 2805266586u, vec3<i32>(var_0.b.x, 1033698918, -1875411016), 1707424184), Struct_3(var_0.d.x, 502999485, Struct_1(vec2<u32>(1531963448u, 2510168421u), 394278754u, vec2<u32>(1079912664u, 345959493u), 280875931u, 1964764715u, vec3<i32>(23861112, 1374036414, -251099038), 637393917), 2032504678, -882953697, Struct_1(vec2<u32>(2857903280u, 2818502953u), 1790166230u, vec2<u32>(2374476481u, 330668653u), 2796407897u, 4200626570u, vec3<i32>(908211181, var_0.b.x, 877212394), var_0.b.x), var_0), vec4<u32>(251464018u, 2245928939u, 3619630392u, 1950011061u))))), var_0);
    }
    loop {
        if ((LOOP_COUNTERS[9u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[9u] = (LOOP_COUNTERS[9u]) + (1u);
        var var_1 = vec3<u32>(2376218339u, 4237279824u, ~(~(~(SAFE_MOD_u32(3632748898u, 2130668449u)))));
    }
    if (any(!(vec3<bool>((true) & (all(var_0.d)), true, !(var_0.a))))) {
        if ((!(all(!(var_0.c)))) || (any(select(!(select(var_0.d.yzw, vec3<bool>(var_0.c.x, var_0.a, var_0.a), false)), vec3<bool>(!(true), false, all(var_0.d)), select(select(var_0.d.zzx, vec3<bool>(true, var_0.c.x, true), false), select(var_0.d.wwy, vec3<bool>(true, true, var_0.d.x), var_0.a), var_0.d.www))))) {
            let var_1 = select(vec4<bool>(var_0.d.x, !(var_0.c.x), (-((var_0.b.x) >> (780829400u))) < (var_0.b.x), (SAFE_DIVIDE_i32(max(-1329415637, var_0.b.x), var_0.b.x)) >= ((select(var_0.b.x, 1715707621, var_0.d.x)) >> (~(938889175u)))), vec4<bool>(all(select(vec4<bool>(var_0.a, false, false, var_0.d.x), var_0.d, !(vec4<bool>(var_0.d.x, false, false, false)))), (var_0.c.x) != (var_0.d.x), true, all(var_0.d)), select(select(select(select(var_0.d, vec4<bool>(var_0.a, false, true, true), false), vec4<bool>(var_0.a, var_0.c.x, false, var_0.d.x), !(vec4<bool>(var_0.c.x, true, var_0.d.x, true))), select(!(var_0.d), var_0.d, select(vec4<bool>(var_0.c.x, true, false, var_0.d.x), var_0.d, var_0.a)), vec4<bool>((1012454898u) < (4173193958u), true, false, var_0.a)), vec4<bool>(true, any(select(vec2<bool>(false, true), var_0.c, var_0.c)), var_0.c.x, true), false));
        }
    }
    var var_1 = select(select(vec4<bool>(any(!(var_0.d.zy)), var_0.d.x, true, (~(908008112)) < (var_0.b.x)), select(vec4<bool>(var_0.a, (529514008u) > (3439478355u), (2182651083u) >= (602719015u), var_0.c.x), var_0.d, any(vec2<bool>(false, true))), select(vec4<bool>(any(var_0.d.www), (var_0.a) || (true), (-759639694) <= (var_0.b.x), all(vec3<bool>(false, var_0.a, true))), vec4<bool>(!(var_0.a), any(var_0.d.wwy), true, all(vec2<bool>(true, true))), vec4<bool>(true, any(var_0.d.wwx), any(var_0.d), var_0.d.x))), !(!(vec4<bool>(!(true), (var_0.b.x) == (var_0.b.x), true, var_0.a))), select(select(vec4<bool>(!(true), var_0.a, (612555557) < (-1783089717), any(var_0.d.xw)), vec4<bool>(all(vec3<bool>(var_0.a, false, true)), !(var_0.c.x), true, (true) != (true)), ((1546659245u) << (2122201409u)) != (755840670u)), var_0.d, !(var_0.d)));
    var var_2 = select(SAFE_MINUS_vec4_i32(vec4<i32>(select(SAFE_MOD_i32(1083604180, var_0.b.x), SAFE_DIVIDE_i32(var_0.b.x, var_0.b.x), !(true)), var_0.b.x, var_0.b.x, select(var_0.b.x, (var_0.b.x) & (160188627), (var_1.x) | (var_0.a))), max((-(vec4<i32>(var_0.b.x, var_0.b.x, var_0.b.x, -2016958859))) & (min(vec4<i32>(var_0.b.x, var_0.b.x, -2029299782, 52399883), vec4<i32>(var_0.b.x, -1635658298, 1114170271, var_0.b.x))), min(abs(vec4<i32>(var_0.b.x, var_0.b.x, -973093668, -1157889256)), vec4<i32>(var_0.b.x, var_0.b.x, var_0.b.x, var_0.b.x)))), vec4<i32>(-(1483097857), 849049641, select(SAFE_MINUS_i32(-(var_0.b.x), var_0.b.x), clamp(max(var_0.b.x, -1308780524), ~(var_0.b.x), ~(var_0.b.x)), var_0.a), 504328406), vec4<bool>(all(select(!(var_1), !(var_0.d), false)), var_1.x, (SAFE_PLUS_i32(-183200385, dot(var_0.b, var_0.b))) > ((-344024426) >> (~(1170244540u))), (SAFE_TIMES_u32((1440829047u) | (4184583378u), SAFE_MINUS_u32(1291198455u, 1934443771u))) < (func_4(Struct_1(vec2<u32>(2571240785u, 1163393737u), 201003506u, vec2<u32>(1575069567u, 179438981u), 1429493132u, 351582131u, vec3<i32>(-589435895, var_0.b.x, -110139502), 1027198210)))));
    return (var_0.c.x) == (!(((-1674332879) ^ (-1221445453)) == (((-748622161) | (var_0.b.x)) >> ((3344560111u) >> (299489105u)))));
}

@stage(compute)
@workgroup_size(1)
fn main() {
    if (func_1(!(((max(4157234642u, 1204522952u)) < (min(1157183880u, 2293246665u))) != (!(all(vec2<bool>(false, false))))), !(false), (select(vec2<u32>(SAFE_MINUS_u32(3850279293u, 2126943604u), 3831047827u), vec2<u32>(604403751u, abs(2299593865u)), !((true) | (false)))) | (~(SAFE_TIMES_vec2_u32(vec2<u32>(731676825u, 2322763367u), (vec2<u32>(3329710115u, 1030975487u)) << (vec2<u32>(3749916605u, 2446112126u))))), Struct_3(all(select(vec2<bool>(false, false), !(vec2<bool>(false, false)), vec2<bool>(false, false))), -(dot(SAFE_MOD_vec3_i32(vec3<i32>(268755583, -1319497744, 1046101299), vec3<i32>(2076759658, 1572923292, 2061385561)), max(vec3<i32>(642422911, 1097412562, -1065531304), vec3<i32>(216735085, 541216049, 1217906638)))), Struct_1(abs(~(vec2<u32>(3672779357u, 2814024263u))), 2539948925u, SAFE_MOD_vec2_u32(select(vec2<u32>(1302741491u, 2396713488u), vec2<u32>(455353161u, 623302168u), true), vec2<u32>(3232549181u, 3704158314u)), clamp(SAFE_MINUS_u32(2295994599u, 3257663565u), ~(3758486369u), select(3570897191u, 2341916730u, false)), ~(dot(vec2<u32>(2446383283u, 3995754186u), vec2<u32>(1740599449u, 2124693388u))), (vec3<i32>(-1006653185, 1761578438, 308403370)) << ((vec3<u32>(2665991290u, 671557869u, 187133031u)) << (vec3<u32>(1338674012u, 3312606253u, 727980566u))), -802833840), 1654918132, -(-(SAFE_MOD_i32(-359871828, 1390554501))), Struct_1(abs(~(vec2<u32>(443483105u, 2423513242u))), ~(~(785727028u)), vec2<u32>(SAFE_PLUS_u32(998385209u, 3718085937u), (3664041423u) & (2402633691u)), 3954018108u, ~(select(1974218312u, 1954873561u, true)), vec3<i32>(-728807377, dot(vec3<i32>(-1638378398, 2026200521, 1082639722), vec3<i32>(-338753171, -2139862107, 1502590295)), -2124636902), (573920818) | (~(-1989333216))), func_3(Struct_1(~(vec2<u32>(3490121920u, 560357205u)), SAFE_MINUS_u32(902071125u, 1231886100u), SAFE_DIVIDE_vec2_u32(vec2<u32>(4229861837u, 3206370436u), vec2<u32>(1307508862u, 3893078436u)), ~(3332658867u), 2667095696u, ~(vec3<i32>(-1628720651, -1553806972, -1511930430)), dot(vec2<i32>(1440403675, 1729541684), vec2<i32>(877736209, 635003628))))))) {
        var var_0 = SAFE_DIVIDE_vec3_u32(~(select(vec3<u32>(2456588299u, dot(vec2<u32>(1256534728u, 4099624764u), vec2<u32>(1585007896u, 3732201371u)), ~(2359864436u)), vec3<u32>(~(570459164u), dot(vec3<u32>(1605992400u, 4222813513u, 328533432u), vec3<u32>(606695594u, 3890244794u, 3889111723u)), dot(vec2<u32>(4081802221u, 36843076u), vec2<u32>(3324598428u, 4208768378u))), select(vec3<bool>(false, false, false), select(vec3<bool>(true, false, true), vec3<bool>(false, true, false), false), select(vec3<bool>(false, false, false), vec3<bool>(false, false, true), true)))), (~(vec3<u32>(~(3815375435u), dot(vec3<u32>(3453001488u, 575389813u, 1037120190u), vec3<u32>(3980140755u, 2245307418u, 1326649788u)), (95937183u) ^ (3533280373u)))) ^ (SAFE_DIVIDE_vec3_u32(~(~(vec3<u32>(2286826739u, 118034534u, 1365479172u))), ~(abs(vec3<u32>(2394363481u, 2394829867u, 588254072u))))));
    }
    if (all(!(!(select(vec3<bool>(true, true, true), select(vec3<bool>(true, false, false), vec3<bool>(false, true, true), vec3<bool>(false, false, false)), true))))) {
        if ((any(!(!(select(vec2<bool>(false, false), vec2<bool>(true, true), vec2<bool>(true, true)))))) != ((!(!(!(true)))) & (!(true)))) {
            loop {
                if ((LOOP_COUNTERS[0u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[0u] = (LOOP_COUNTERS[0u]) + (1u);
                let var_0 = ~(~(~(~(4251135771u))));
            }
        }
    }
    if (!(true)) {
        if (((-(min(SAFE_TIMES_i32(1848232366, -459612667), ~(270213341)))) > (select(abs((-304395251) ^ (690236611)), ~(~(-87311255)), (!(false)) != (true)))) & (((4052509002u) & (~(~(221361937u)))) >= (SAFE_MINUS_u32(2546852191u, min(3424056620u, func_2(Struct_1(vec2<u32>(1131931267u, 4039951416u), 1982968286u, vec2<u32>(3540219220u, 3176727320u), 3116779976u, 1608472203u, vec3<i32>(1744059276, -1324699876, -1745797569), 1124429577), Struct_3(false, -453988565, Struct_1(vec2<u32>(146637999u, 1785628353u), 3767484072u, vec2<u32>(4126737737u, 816568612u), 2477889101u, 4189602661u, vec3<i32>(-923169716, 765541634, 2001477155), 256886677), 1582431172, -1742591963, Struct_1(vec2<u32>(3382823638u, 756770199u), 2351105639u, vec2<u32>(2161693838u, 439010680u), 1889612192u, 990735361u, vec3<i32>(-315588455, -1851968287, 1831882253), 746742323), Struct_2(false, vec2<i32>(-963396779, 235137944), vec2<bool>(true, true), vec4<bool>(false, true, false, true))), vec4<u32>(1095997758u, 1767648500u, 3024121166u, 3963216376u))))))) {
            var var_0 = func_3(Struct_1(~(vec2<u32>(abs(293313197u), 1046295096u)), ~(~((2425663094u) | (3440526002u))), vec2<u32>(clamp(SAFE_TIMES_u32(4146825787u, 2037939912u), SAFE_DIVIDE_u32(3632737336u, 1388730237u), 3621695609u), 26777909u), func_4(Struct_1(~(vec2<u32>(3629222804u, 4163984798u)), clamp(2450120524u, 583806430u, 820606829u), SAFE_MINUS_vec2_u32(vec2<u32>(1280224303u, 2810379735u), vec2<u32>(2599680736u, 3551715296u)), SAFE_TIMES_u32(393219787u, 3094197650u), 2205373333u, max(vec3<i32>(-1343138061, -1153763870, 28796196), vec3<i32>(-1393001985, 1540335603, -959943056)), dot(vec3<i32>(-747545086, -1909894668, -725565943), vec3<i32>(-1983194267, 251579192, -1938224468)))), 3997030837u, ~(clamp(abs(vec3<i32>(1497896140, 923274447, -921314465)), SAFE_TIMES_vec3_i32(vec3<i32>(1013518980, 131678690, -894138366), vec3<i32>(1556786825, -1649586511, 1519104002)), (vec3<i32>(1925258166, -1151929090, 2122003015)) ^ (vec3<i32>(-1167618989, 1046366252, -1685214184)))), -(dot(-(vec4<i32>(1814960886, 1239778902, 2044001182, -1381482744)), vec4<i32>(1441093975, 947915776, -27231398, 1803598037)))));
        }
    }
    if (true) {
        let var_0 = vec4<i32>(dot(SAFE_TIMES_vec4_i32(min(-(vec4<i32>(-1991896585, -1051398835, -54509543, 1665046175)), select(vec4<i32>(-1176990295, 615748770, 489363960, 1391732246), vec4<i32>(-1408651724, 654527828, -501629624, 440580929), vec4<bool>(false, true, false, false))), -(~(vec4<i32>(-1635918563, -1290219648, 881848183, 169230084)))), ~(select(vec4<i32>(1000628066, 272649424, -1851536070, -1977255382), vec4<i32>(-1290758242, 753289954, 1143597594, -1440505680), select(vec4<bool>(false, true, true, false), vec4<bool>(false, true, false, true), true)))), ~(515029110), -((-1467341831) >> (388228639u)), -(-1094641684));
    }
    var var_0 = clamp(~(-(dot(vec4<i32>(-1716851279, -1813279808, -1707014020, 1031248532), -(vec4<i32>(259163435, -1698129790, 2138311181, -415736462))))), ~(dot(vec3<i32>(-(1338771583), 1632231563, SAFE_MINUS_i32(-1077942804, 1809959655)), ~(max(vec3<i32>(1830229280, -703182032, -1751434661), vec3<i32>(1674815593, -654837166, -105518159))))), (-428505838) >> (max(SAFE_DIVIDE_u32(~(4093936258u), 1256032103u), 4148961508u)));
    output.value = input.value;
    output.value = min(dot(vec3<u32>((~(2355295804u)) ^ (2256590399u), 1496785516u, 4141118986u), ~((max(vec3<u32>(1595300491u, 4066072070u, 985234366u), vec3<u32>(1757963692u, 186181084u, 1017695419u))) | (~(vec3<u32>(2559398839u, 2741532993u, 1210028363u))))), dot(min(select(~(vec2<u32>(3694394529u, 3203757779u)), ~(vec2<u32>(2153864903u, 3767051050u)), !(true)), clamp(SAFE_MINUS_vec2_u32(vec2<u32>(2220133441u, 2031277314u), vec2<u32>(983184671u, 1432384098u)), vec2<u32>(716625470u, 2982867496u), abs(vec2<u32>(2128628678u, 4205057592u)))), SAFE_DIVIDE_vec2_u32(SAFE_TIMES_vec2_u32(~(vec2<u32>(1129819818u, 1185319269u)), vec2<u32>(675000875u, 1997251415u)), abs(select(vec2<u32>(2852643288u, 902956277u), vec2<u32>(2685851644u, 3257597095u), false)))));
}

