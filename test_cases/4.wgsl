// {"resources":[{"kind":"UniformBuffer","group":0,"binding":0,"size":4,"init":[93,11,237,68]},{"kind":"StorageBuffer","group":0,"binding":1,"size":4,"init":null}]}
// Seed: 5190154919564049088

var<private> LOOP_COUNTERS: array<u32, 12>;

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
    a: vec3<i32>;
    b: i32;
    c: vec2<bool>;
    d: vec4<i32>;
    e: i32;
    f: i32;
    g: vec3<i32>;
};

struct Struct_2 {
    a: vec4<bool>;
    b: vec2<u32>;
    c: u32;
    d: Struct_1;
    e: vec2<i32>;
    f: i32;
    g: i32;
    h: Struct_1;
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

fn SAFE_PLUS_vec2_u32(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y));
}

fn SAFE_PLUS_vec4_u32(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
    return vec4<u32>(SAFE_PLUS_u32(a.x, b.x), SAFE_PLUS_u32(a.y, b.y), SAFE_PLUS_u32(a.z, b.z), SAFE_PLUS_u32(a.w, b.w));
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

fn SAFE_TIMES_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_TIMES_i32(a.x, b.x), SAFE_TIMES_i32(a.y, b.y));
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

fn SAFE_DIVIDE_vec2_i32(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
    return vec2<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y));
}

fn SAFE_DIVIDE_vec3_i32(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y), SAFE_DIVIDE_i32(a.z, b.z));
}

fn SAFE_DIVIDE_vec4_i32(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
    return vec4<i32>(SAFE_DIVIDE_i32(a.x, b.x), SAFE_DIVIDE_i32(a.y, b.y), SAFE_DIVIDE_i32(a.z, b.z), SAFE_DIVIDE_i32(a.w, b.w));
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

fn func_4(arg_0: vec2<i32>, arg_1: Struct_2, arg_2: Struct_1, arg_3: bool) -> bool {
    if (!(true)) {
        loop {
            if ((LOOP_COUNTERS[4u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[4u] = (LOOP_COUNTERS[4u]) + (1u);
            loop {
                if ((LOOP_COUNTERS[5u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[5u] = (LOOP_COUNTERS[5u]) + (1u);
                {
                    let var_0 = Struct_2(!(vec4<bool>(!(!(false)), false, true, !(true))), ~(~((select(vec2<u32>(1597610151u, 2999215523u), vec2<u32>(3697006235u, 215091054u), vec2<bool>(true, false))) ^ (abs(vec2<u32>(1277524237u, 2394682022u))))), dot(SAFE_TIMES_vec2_u32(select(vec2<u32>(3317520161u, 4008392087u), abs(vec2<u32>(3706097056u, 1509974619u)), (false) || (false)), min(vec2<u32>(3365462892u, 4257632628u), vec2<u32>(4072177103u, 3014876989u))), ~(clamp(vec2<u32>(2425414337u, 517588502u), ~(vec2<u32>(2540925538u, 93159821u)), ~(vec2<u32>(3176580515u, 548803181u))))), Struct_1(-(~((vec3<i32>(1130069745, -676516811, -1503309140)) >> (vec3<u32>(8196109u, 4197737615u, 4259373363u)))), (dot(select(vec4<i32>(-666390108, -895063450, 1593718244, -910537918), vec4<i32>(808161924, -6591678, -772830060, -286254504), vec4<bool>(true, false, true, false)), -(vec4<i32>(1544212750, 1693558370, 1691989399, 1534383006)))) ^ (-1126707016), select(vec2<bool>(all(vec4<bool>(true, false, true, false)), (1336193314u) >= (757582872u)), !(vec2<bool>(false, false)), false), SAFE_PLUS_vec4_i32(select(vec4<i32>(-1460617428, 1066915954, 20405264, -1679154125), select(vec4<i32>(989927013, 470053261, 1870682698, -171152795), vec4<i32>(770565641, 1185089704, -900516142, 1415893335), vec4<bool>(false, true, false, true)), select(vec4<bool>(false, false, true, true), vec4<bool>(true, true, false, true), false)), SAFE_DIVIDE_vec4_i32(max(vec4<i32>(-1071477556, 1292034378, 1443938860, 182516007), vec4<i32>(-1426632259, 1458820349, -177407265, 422031955)), vec4<i32>(-1882011713, 1507103506, -1972071526, -1760294102))), dot(~(vec2<i32>(1056821854, 528320853)), abs(-(vec2<i32>(-1840250818, 691633914)))), 809240840, select(vec3<i32>(-(450842321), -927689902, (856701122) << (2451199971u)), ~(vec3<i32>(1143534426, -894589927, -799266258)), all(select(vec2<bool>(true, true), vec2<bool>(false, false), true)))), vec2<i32>(1099362453, -1638553112), -(-(-(dot(vec3<i32>(-1217828508, -1465506922, -1238498940), vec3<i32>(-1941829104, -1863857905, 1710578801))))), SAFE_MINUS_i32(SAFE_MOD_i32((~(765744374)) | (~(2140939941)), -(717349544)), clamp(~((2127198631) ^ (-1017300206)), -(-(839622730)), 1005383324)), Struct_1(~(max(vec3<i32>(-1017767305, -733507133, -1383821629), ~(vec3<i32>(654758607, 670016867, -1689264755)))), dot(~(vec2<i32>(744545993, -558347939)), vec2<i32>(clamp(405953969, -2092079476, -258246694), clamp(987669448, -864933757, 1274491942))), vec2<bool>(!((true) & (true)), false), min(-(-(vec4<i32>(-211121914, -1122362271, 1108180896, 637508804))), (max(vec4<i32>(-1826936530, 1683410957, 1619184597, -473490900), vec4<i32>(-1039731914, 1821907423, -628570838, 1363436518))) ^ (SAFE_MOD_vec4_i32(vec4<i32>(1449242864, 997022251, -484118550, 2002214148), vec4<i32>(-2011354632, 130569184, -102348145, -1875477220)))), 1737627060, SAFE_MOD_i32(SAFE_MOD_i32(SAFE_PLUS_i32(-475672210, -1882010802), (1309278978) >> (450463493u)), 1945505084), clamp(SAFE_PLUS_vec3_i32(SAFE_MINUS_vec3_i32(vec3<i32>(-102429139, -1823448632, -1699998569), vec3<i32>(178636668, 1038910169, 393163315)), ~(vec3<i32>(1232955017, 1955317958, 359115218))), ~(-(vec3<i32>(-170917522, 374590629, 343191191))), max(~(vec3<i32>(-2064629203, -1467570243, 1402020926)), (vec3<i32>(1252390853, -1103107574, 252404832)) >> (vec3<u32>(915855066u, 1682460363u, 825393908u))))));
                }
            }
        }
    }
    if (true) {
        loop {
            if ((LOOP_COUNTERS[6u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[6u] = (LOOP_COUNTERS[6u]) + (1u);
            let var_0 = Struct_1(vec3<i32>(-1174428010, select(-573695844, ~((-915991955) << (4237553829u)), (all(vec4<bool>(true, false, true, true))) || (!(false))), (1030449240) | (1915634616)), abs(350227), !(vec2<bool>(!(any(vec3<bool>(true, false, false))), true)), clamp(max(~(SAFE_DIVIDE_vec4_i32(vec4<i32>(-2143672288, 668736860, -153241679, 1658827504), vec4<i32>(-1559216191, -621712983, -504038461, 1977349347))), max((vec4<i32>(-1259969531, -1464843903, 429244495, -1464890709)) >> (vec4<u32>(1621588298u, 1767634682u, 499970016u, 657124983u)), (vec4<i32>(-589251174, 134248879, 1497012955, 1765788021)) << (vec4<u32>(388032237u, 3911503933u, 2050578048u, 1735174416u)))), clamp(-(vec4<i32>(-626878027, -1806577162, -164633027, -727772935)), min(SAFE_TIMES_vec4_i32(vec4<i32>(-1890364078, -1421549633, 409553834, 1944388001), vec4<i32>(-782975908, 854203165, 1764144732, -763156438)), select(vec4<i32>(75589710, 1535404254, -660564175, 163554650), vec4<i32>(5780149, 977107659, -1297023899, 819870634), vec4<bool>(true, true, false, false))), min(SAFE_TIMES_vec4_i32(vec4<i32>(-1042882614, -484815008, -155379436, 1342140329), vec4<i32>(877970129, 1709620517, 128053093, -539880537)), abs(vec4<i32>(-1041479622, -1280767590, -1063247625, 129275929)))), vec4<i32>((1416604000) & (-(-2061411908)), -701784544, SAFE_PLUS_i32((-1230695892) | (-1580314706), 89022607), (min(445397916, -377898465)) >> (~(2444474518u)))), 954985394, 1815390695, min(SAFE_MINUS_vec3_i32((max(vec3<i32>(-1984626207, 1902560796, 1669824277), vec3<i32>(231432246, -695962361, -2022936682))) << (clamp(vec3<u32>(1641730699u, 914575072u, 2062701268u), vec3<u32>(3706993217u, 761386743u, 1016715919u), vec3<u32>(1398300280u, 389272101u, 3607409457u))), (vec3<i32>(1874969055, -449144356, -1726788065)) ^ (clamp(vec3<i32>(-820038636, 1405133850, -1144647225), vec3<i32>(-36891474, 1420101513, 1710258540), vec3<i32>(390607650, -1103056697, -1091676378)))), vec3<i32>(-1032388061, ~(~(2001050881)), clamp((-80133093) & (771077157), dot(vec3<i32>(1660088554, 91090916, 1921840912), vec3<i32>(313358586, 2052462120, 1088652704)), SAFE_MINUS_i32(-1483622235, 1021385353)))));
        }
    }
    if (select(select(!(any(select(vec3<bool>(true, false, true), vec3<bool>(true, true, false), false))), any(vec2<bool>(true, (false) & (true))), (!(all(vec4<bool>(true, false, false, false)))) == ((~(1690777020)) != ((914076978) << (2759275010u)))), ((!(false)) & (!(false))) | ((SAFE_MOD_i32(dot(vec2<i32>(-68619989, -1816245593), vec2<i32>(1657253198, 1823711820)), -1567079879)) > (max(SAFE_DIVIDE_i32(1645390235, 1564726916), 1767689458))), (-(~(-(2042029136)))) <= ((dot((vec2<i32>(115132561, 412727721)) | (vec2<i32>(833652986, 2007085543)), (vec2<i32>(1031933397, 343071210)) | (vec2<i32>(1849807867, -919093535)))) | (~(SAFE_PLUS_i32(1980925208, 271534520)))))) {
        if ((min((abs(min(3554127295u, 418023072u))) << (select((1039707595u) | (1978319135u), ~(2624024739u), any(vec2<bool>(true, true)))), clamp(max(~(1253132150u), ~(2227275318u)), SAFE_MOD_u32(SAFE_MINUS_u32(1958866994u, 3055467256u), 2055802179u), (SAFE_MINUS_u32(2309030411u, 2583062935u)) << (1586954164u)))) == (max(128749839u, ~((3613630526u) << (SAFE_PLUS_u32(546755657u, 1957250771u)))))) {
            var var_0 = Struct_2(vec4<bool>(true, !(!(!(true))), any(!(vec4<bool>(true, true, true, true))), true), vec2<u32>((SAFE_TIMES_u32(dot(vec2<u32>(1348206569u, 3128899521u), vec2<u32>(2147408019u, 3117609315u)), 2090561402u)) << (((1861705133u) & (2081636775u)) ^ (SAFE_DIVIDE_u32(1332872980u, 2079191791u))), SAFE_PLUS_u32(dot(vec3<u32>(1943582565u, 150685883u, 904818416u), vec3<u32>(3504996145u, 3186924518u, 1743339618u)), 3879863429u)), (dot(clamp((vec4<u32>(3032969787u, 1787773506u, 949134855u, 779121942u)) << (vec4<u32>(2350097182u, 396135383u, 756276177u, 1787061721u)), max(vec4<u32>(898204127u, 2806242450u, 2683752800u, 4029495895u), vec4<u32>(1615052617u, 357708570u, 1011881432u, 558102512u)), ~(vec4<u32>(3764085022u, 3501254522u, 2257086846u, 5108072u))), (vec4<u32>(333423765u, 3163703872u, 3787286957u, 1534197311u)) << (vec4<u32>(2996581445u, 3816695759u, 4054593955u, 1403196177u)))) & (dot(~(select(vec4<u32>(1495136261u, 4124171113u, 1016612419u, 543957623u), vec4<u32>(3384214436u, 3117633819u, 1507648747u, 3263709550u), false)), ~(vec4<u32>(3490027836u, 1073975875u, 412866225u, 3805437169u)))), Struct_1(-(vec3<i32>((491055160) >> (3661331917u), ~(35516712), clamp(-574470087, -1838465095, 152300156))), dot(-(-(vec4<i32>(-820916646, 1696173879, -1804024564, 735490765))), vec4<i32>(~(2116662866), select(1537658597, 1239964681, false), (1425374410) ^ (1890701368), select(43348926, 1211678992, true))), !(vec2<bool>(all(vec4<bool>(true, true, false, true)), true)), -(vec4<i32>(select(939098033, 2069449678, false), SAFE_PLUS_i32(1050389038, 790016199), clamp(-626965029, -1076607663, -778112658), 1864328497)), -193257799, min(-(max(1140500688, -1466792701)), dot(~(vec3<i32>(-1350270398, -1708217463, -246327382)), abs(vec3<i32>(968077339, -1759091815, -567540238)))), -(~(select(vec3<i32>(654138030, 335012079, -1433892927), vec3<i32>(-1172212639, -565783850, 811222140), false)))), ~(SAFE_PLUS_vec2_i32(-(-(vec2<i32>(439116091, 1311988757))), max(SAFE_MINUS_vec2_i32(vec2<i32>(-172571240, -733402155), vec2<i32>(556764962, -134489478)), vec2<i32>(-2097133257, -409464312)))), SAFE_MINUS_i32(-316149769, select((-(-629134769)) << (~(3930295178u)), (~(-677703507)) >> (dot(vec3<u32>(691965244u, 2463026893u, 3739684217u), vec3<u32>(1425945820u, 2478799798u, 2181172467u))), !(!(true)))), dot(SAFE_TIMES_vec3_i32(~(~(vec3<i32>(1118706677, 1024544397, 481250245))), vec3<i32>(dot(vec4<i32>(1797851990, -705165101, 38659457, 1506706662), vec4<i32>(948526372, -1590428473, 1305493365, 1470174613)), dot(vec4<i32>(-805648772, 1462823586, -1168893243, -1386507261), vec4<i32>(-1948172242, -660642887, -2146690326, 537718903)), -446814084)), vec3<i32>(SAFE_MOD_i32(~(-144922463), ~(-334653260)), select(dot(vec3<i32>(889081929, 2081350050, -725704734), vec3<i32>(1882189536, 681183890, 221510286)), select(-1854493737, 1162489956, false), (1501411479u) != (3585388244u)), 1192953636)), Struct_1(SAFE_TIMES_vec3_i32(vec3<i32>(SAFE_MINUS_i32(-1113909196, 1443318929), (-2044946688) << (848076512u), 1664582762), -(SAFE_MINUS_vec3_i32(vec3<i32>(516013062, 1311407278, -821411164), vec3<i32>(-1909827001, -895310586, 1132911332)))), (SAFE_PLUS_i32(clamp(-1842226954, 2086506108, -1692838409), -2023699343)) & (~(SAFE_DIVIDE_i32(1381958094, -1750531261))), vec2<bool>(((-2118021218) & (571906502)) <= (2107588785), !(all(vec2<bool>(false, true)))), vec4<i32>(~((1029238499) ^ (1135159555)), (-(-648758466)) << (~(3371809365u)), -(1436296104), 2033280792), ~((dot(vec3<i32>(-615678704, 137595499, -1819904641), vec3<i32>(-2142395529, -1062062284, -438604540))) | (SAFE_MINUS_i32(1925635197, 1522377461))), abs(dot(abs(vec2<i32>(235691649, -1557848540)), (vec2<i32>(-2063347278, -9781515)) & (vec2<i32>(214598727, 837170086)))), vec3<i32>(((-413048718) | (-121527693)) << (dot(vec3<u32>(2424308561u, 1986392676u, 2136444629u), vec3<u32>(2306761431u, 2211580664u, 4129848057u))), (SAFE_MINUS_i32(1804537321, 849625930)) | (1473507297), ((874330774) >> (855144084u)) ^ (654248863))));
        }
    }
    let var_0 = vec3<bool>(true, !(true), !(!(true)));
    var var_1 = -(-634741703);
    var_1 = -(-(-1029257458));
    return all(vec3<bool>(true, (!(!(var_0.x))) & (all(select(vec4<bool>(var_0.x, var_0.x, var_0.x, true), vec4<bool>(false, false, var_0.x, var_0.x), true))), ((var_0.x) & (true)) | ((true) & (true))));
}

fn func_3(arg_0: vec3<bool>, arg_1: Struct_1, arg_2: vec3<i32>) -> i32 {
    loop {
        if ((LOOP_COUNTERS[7u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[7u] = (LOOP_COUNTERS[7u]) + (1u);
        let var_0 = select(!(vec2<bool>(((-73884835) >> (2096602135u)) <= (1799981318), func_4(vec2<i32>(2070097263, 2103171073), Struct_2(vec4<bool>(false, false, false, false), vec2<u32>(584924000u, 2817387671u), 1789026863u, Struct_1(vec3<i32>(-258369673, 1556043098, -1051446879), 703231598, vec2<bool>(false, true), vec4<i32>(-858176599, 115584226, 463285512, -2061312396), -461064512, -839634247, vec3<i32>(1345996040, -850243815, -983003042)), vec2<i32>(417623757, -1238284119), 891020477, -302815516, Struct_1(vec3<i32>(1114172995, 296598564, -477045953), -132676088, vec2<bool>(true, true), vec4<i32>(1066361243, -1223575411, 1928771340, 203618958), 946834864, -253339430, vec3<i32>(-55129900, 102879931, 756625140))), Struct_1(vec3<i32>(-120524948, -1902319237, -405039606), -1861742224, vec2<bool>(false, false), vec4<i32>(-1495505694, -696166179, -297741084, -1660141749), 1219855593, 1059780284, vec3<i32>(-1845252346, -369680907, 830269207)), false))), vec2<bool>((((1370774670) >> (308518524u)) << (SAFE_PLUS_u32(2815809743u, 2198119592u))) != (-1384386392), ((dot(vec4<u32>(1432138966u, 702321551u, 655260611u, 740600389u), vec4<u32>(4065623818u, 1095822271u, 4087978843u, 3908261653u))) << (3699424791u)) > ((SAFE_PLUS_u32(995073491u, 1571356444u)) & (select(640569823u, 3193056137u, true)))), vec2<bool>(all(select(select(vec4<bool>(true, false, false, false), vec4<bool>(true, true, false, true), vec4<bool>(false, true, false, true)), select(vec4<bool>(true, true, false, true), vec4<bool>(true, false, false, false), true), (false) | (true))), any(!(select(vec2<bool>(false, false), vec2<bool>(false, false), false)))));
    }
    var var_0 = !(select(!(!(!(vec3<bool>(false, false, true)))), select(!(!(vec3<bool>(false, true, false))), !(!(vec3<bool>(true, false, false))), false), !(!(all(vec3<bool>(false, true, true))))));
    if (!(false)) {
        loop {
            if ((LOOP_COUNTERS[8u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[8u] = (LOOP_COUNTERS[8u]) + (1u);
            var var_1 = select(!(var_0.zy), select(select(vec2<bool>(any(vec4<bool>(true, var_0.x, false, var_0.x)), true), select(var_0.yz, vec2<bool>(false, var_0.x), vec2<bool>(false, var_0.x)), vec2<bool>((-825584108) >= (1962665577), var_0.x)), select(!(select(var_0.xx, vec2<bool>(var_0.x, var_0.x), true)), vec2<bool>(any(vec4<bool>(var_0.x, var_0.x, true, true)), var_0.x), vec2<bool>(true, !(var_0.x))), !(var_0.xy)), !(vec2<bool>(false, (min(175476626u, 3565171162u)) < (dot(vec4<u32>(227176147u, 2835005921u, 2815868489u, 22583413u), vec4<u32>(713127442u, 3687830277u, 2330572783u, 3347140306u))))));
        }
    }
    var var_1 = max(clamp(~(2647108332u), (162967712u) << (clamp(dot(vec2<u32>(915373307u, 738180628u), vec2<u32>(960299307u, 22247320u)), ~(1771837740u), ~(1684178149u))), select(dot(max(vec2<u32>(3066200824u, 3296466297u), vec2<u32>(1397818693u, 3027078734u)), vec2<u32>(594271996u, 1753115502u)), 3544138134u, true)), dot(SAFE_PLUS_vec4_u32(~(~(vec4<u32>(2330700376u, 993339058u, 3355338547u, 59178570u))), min(~(vec4<u32>(3017908283u, 3299033501u, 1291762089u, 1402256604u)), min(vec4<u32>(2458332692u, 3668226525u, 3185582015u, 186237368u), vec4<u32>(2603586810u, 2949359400u, 1670891638u, 4065479861u)))), vec4<u32>(1795760153u, select(SAFE_MOD_u32(476780071u, 3297873338u), select(1298341387u, 189851391u, false), (2498216216u) >= (2210689576u)), SAFE_MINUS_u32((2180124840u) | (1591527063u), SAFE_MINUS_u32(2958493189u, 3145592539u)), dot(vec4<u32>(2263764725u, 1807274066u, 342335251u, 3030352592u), ~(vec4<u32>(1945407602u, 3150032668u, 333800023u, 1945921532u))))));
    loop {
        if ((LOOP_COUNTERS[9u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[9u] = (LOOP_COUNTERS[9u]) + (1u);
        var_0 = !(select(var_0, var_0, vec3<bool>(true, false, (~(-1044076297)) == (1080249353))));
    }
    var_1 = 3927904197u;
    return abs(-970353514);
}

fn func_2(arg_0: vec3<u32>, arg_1: u32, arg_2: vec2<bool>) -> bool {
    let var_0 = Struct_1(vec3<i32>(~(1484324756), ~(177936316), ~((dot(vec4<i32>(666722500, -540828380, -1972586993, -1291522579), vec4<i32>(-408559953, -2062165530, 1352914230, 1678730562))) >> (~(1510223878u)))), -1771148191, !(!(!(vec2<bool>(false, true)))), abs(vec4<i32>(1635070358, (SAFE_PLUS_i32(-143305117, -154534494)) ^ (995575965), dot(SAFE_MINUS_vec4_i32(vec4<i32>(147047673, -1534091161, 936754092, 810961437), vec4<i32>(1575505720, 1539423395, -1492835068, 4174314)), select(vec4<i32>(2088828658, -2021511243, -2026251403, 405621633), vec4<i32>(1902765330, -2042425882, 692138338, -1031143213), vec4<bool>(true, true, false, true))), (SAFE_MINUS_i32(-731564536, 250937863)) | (~(1703409329)))), ~(((dot(vec4<i32>(1755716446, -536400265, -2038684091, -1524851920), vec4<i32>(-1291889919, 838386271, -342764845, -1072206157))) | ((-1947239380) ^ (505490828))) >> (select(dot(vec3<u32>(702277680u, 1115790250u, 3863387349u), vec3<u32>(3273776286u, 4047909017u, 3361416280u)), dot(vec2<u32>(1121121805u, 372811497u), vec2<u32>(451187163u, 1146842940u)), (1067599101u) == (4249083296u)))), func_3(vec3<bool>(true, true, !((1450815133u) > (1443158725u))), Struct_1(vec3<i32>(-1724239509, ~(-2062327022), clamp(1635511886, 1026939069, 874544705)), (min(-1499172686, 1891186617)) | (func_3(vec3<bool>(false, true, true), Struct_1(vec3<i32>(1874178284, -641873854, -1957638247), -797963432, vec2<bool>(true, false), vec4<i32>(2009668310, -75020791, -1112317254, 2019686625), 891802849, 952815299, vec3<i32>(-1734155741, 1458899777, -705881121)), vec3<i32>(633508278, 1973018976, -1884561926))), !(vec2<bool>(false, true)), -(~(vec4<i32>(557626359, -1413558371, 1361532762, 1621060708))), (380867845) ^ (select(1332243458, -1685534752, true)), -976445695, min(vec3<i32>(-1377333436, -372308391, -629656496), (vec3<i32>(-1698998203, -870060355, 1922288698)) << (vec3<u32>(911703734u, 3009302179u, 1896217257u)))), clamp((~(vec3<i32>(-1245053374, 431259639, 1865781020))) & (vec3<i32>(-1352507621, -1162767445, 2037300216)), vec3<i32>(~(-2022449534), (1632687364) << (1714015915u), SAFE_DIVIDE_i32(-263824323, -871587091)), SAFE_TIMES_vec3_i32(vec3<i32>(-1333590763, 2096052057, -1523651694), (vec3<i32>(824337751, -2255192, -91722038)) ^ (vec3<i32>(97037078, 1874330068, 1046397545))))), vec3<i32>(-1407083734, 1515541275, -495498800));
    var var_1 = Struct_2(vec4<bool>(func_4(var_0.g.zx, Struct_2(!(vec4<bool>(true, var_0.c.x, false, false)), vec2<u32>(3601875441u, 3422861627u), (3026054975u) >> (2963823975u), var_0, select(var_0.g.xy, vec2<i32>(var_0.e, 489324571), var_0.c.x), var_0.e, var_0.d.x, var_0), Struct_1(abs(vec3<i32>(var_0.g.x, -1871397204, var_0.d.x)), SAFE_DIVIDE_i32(-2095797369, var_0.f), !(var_0.c), var_0.d, var_0.e, 1684001774, var_0.g), (3412944281u) == (1385555300u)), var_0.c.x, !(false), false), (~(select(abs(vec2<u32>(3901744440u, 2422307817u)), ~(vec2<u32>(1170952154u, 1908266709u)), !(var_0.c.x)))) >> (SAFE_MOD_vec2_u32(max(select(vec2<u32>(3561980649u, 2337482704u), vec2<u32>(3505984003u, 2229968305u), true), ~(vec2<u32>(2956684246u, 2057535967u))), ~(~(vec2<u32>(4277558573u, 3259819693u))))), ~(3008023933u), var_0, ~(var_0.a.xx), -(var_0.d.x), SAFE_DIVIDE_i32(var_0.g.x, ~(func_3(select(vec3<bool>(true, var_0.c.x, false), vec3<bool>(true, var_0.c.x, var_0.c.x), vec3<bool>(var_0.c.x, false, false)), var_0, (var_0.g) << (vec3<u32>(2499098915u, 2371752309u, 779367671u))))), var_0);
    let var_2 = max(SAFE_PLUS_vec2_u32(vec2<u32>(min((3550201336u) & (2092499714u), (var_1.b.x) & (var_1.b.x)), var_1.c), var_1.b), max(select(var_1.b, min(max(vec2<u32>(var_1.b.x, 579895955u), var_1.b), SAFE_PLUS_vec2_u32(vec2<u32>(var_1.c, var_1.c), vec2<u32>(var_1.b.x, 1975837124u))), false), var_1.b));
    var var_3 = Struct_1((SAFE_MOD_vec3_i32(select(vec3<i32>(1973324030, var_0.g.x, -1538256589), vec3<i32>(-1782241591, -608032155, var_1.h.d.x), var_0.c.x), ((vec3<i32>(-1600640334, -1602908058, var_0.b)) | (vec3<i32>(var_0.g.x, var_1.h.a.x, var_1.d.a.x))) << (~(vec3<u32>(343796688u, 1603930807u, 2721751399u))))) >> (~(~(~(vec3<u32>(var_2.x, 846373824u, 2176775721u))))), 1521222928, var_0.c, min(~(SAFE_TIMES_vec4_i32(vec4<i32>(-2043652395, var_1.e.x, var_1.g, var_1.g), -(vec4<i32>(var_1.f, var_0.f, 183265135, 551860031)))), (-(SAFE_PLUS_vec4_i32(var_0.d, var_0.d))) ^ (select(SAFE_DIVIDE_vec4_i32(var_0.d, vec4<i32>(var_1.d.b, -2038985618, var_1.d.b, var_1.g)), vec4<i32>(var_1.f, 1629861976, var_1.g, 784075466), var_0.c.x))), dot(vec3<i32>(func_3(!(var_1.a.zyz), var_0, vec3<i32>(var_1.e.x, var_0.g.x, -898850766)), -(-1385881985), var_0.d.x), -(var_0.d.zzx)), var_1.g, -(var_0.d.xxy));
    let var_4 = var_1.h;
    let var_5 = var_1.a.xwy;
    var_3 = Struct_1(vec3<i32>(var_0.b, ((SAFE_MINUS_i32(var_0.b, var_1.e.x)) ^ (func_3(vec3<bool>(true, var_3.c.x, false), Struct_1(vec3<i32>(1892964294, 592772916, var_0.g.x), -418632593, var_4.c, var_3.d, var_3.b, 391327560, vec3<i32>(-572257644, -1314100943, var_4.g.x)), var_4.d.zyz))) >> (2415263377u), SAFE_DIVIDE_i32(SAFE_PLUS_i32(-1277504307, -1726957266), 1626852500)), ~((select(var_0.b, max(var_4.e, -1732635185), !(var_4.c.x))) ^ (max(-1182579112, (576907935) << (var_1.c)))), var_0.c, -(vec4<i32>(var_4.a.x, var_1.h.e, var_1.d.g.x, dot(vec3<i32>(1246219581, var_3.f, 1918375838), vec3<i32>(var_1.g, -319519585, var_3.e)))), ~(~(var_3.g.x)), 130247059, SAFE_MOD_vec3_i32(max(((var_3.g) & (var_0.d.xyz)) ^ (~(var_0.g)), select(clamp(var_1.d.a, vec3<i32>(325680702, var_4.e, var_1.f), vec3<i32>(var_3.d.x, var_0.d.x, var_0.g.x)), var_4.g, select(vec3<bool>(var_0.c.x, false, true), var_1.a.zzy, var_1.h.c.x))), SAFE_DIVIDE_vec3_i32(var_3.g, var_4.g)));
    var_1 = var_1;
    return ((-1976001718) >> (var_2.x)) <= (dot(var_3.d.zyy, ~(min(select(var_1.h.g, vec3<i32>(var_3.f, var_3.f, 1037379109), true), vec3<i32>(-1802820016, var_4.e, var_4.a.x)))));
}

fn func_1(arg_0: u32) -> bool {
    if ((2800246956u) < (dot(~(~(vec2<u32>(59503040u, 3946435391u))), ~(vec2<u32>(SAFE_TIMES_u32(2818213400u, 274132303u), ~(1324062779u)))))) {
        if ((abs(1416800993)) >= (~(-(SAFE_PLUS_i32(-(-1470899299), dot(vec4<i32>(-1269756347, -1220277668, 697174187, -818771683), vec4<i32>(2075617086, -1658997092, 1084915813, 184743699))))))) {
            if ((668940203u) > (2963451330u)) {
                if (false) {
                    loop {
                        if ((LOOP_COUNTERS[10u]) >= (1u)) {
                            break;
                        }
                        LOOP_COUNTERS[10u] = (LOOP_COUNTERS[10u]) + (1u);
                        let var_0 = 397296331;
                    }
                }
            }
        }
    }
    var var_0 = Struct_2(vec4<bool>(any(select(vec4<bool>(false, false, true, false), select(vec4<bool>(true, false, false, false), vec4<bool>(false, true, true, true), vec4<bool>(true, false, false, true)), func_2(vec3<u32>(2098216928u, 3209746989u, 315204530u), 3676760218u, vec2<bool>(false, true)))), !(true), all(select(select(vec3<bool>(true, false, true), vec3<bool>(false, false, false), true), !(vec3<bool>(false, true, true)), false)), false), ~(~(~(~(vec2<u32>(1389967217u, 3281713045u))))), 3002796860u, Struct_1(select(select((vec3<i32>(1789566500, -757188148, 1285634559)) << (vec3<u32>(3145083320u, 2082028935u, 1935665627u)), -(vec3<i32>(-680635799, -1392772952, -70812053)), (1055410600u) > (868231927u)), -((vec3<i32>(-2074685678, 95868398, -1496561260)) >> (vec3<u32>(77226298u, 2379039604u, 1979857691u))), select(vec3<bool>(false, false, true), select(vec3<bool>(true, true, false), vec3<bool>(true, false, false), true), vec3<bool>(true, true, true))), SAFE_TIMES_i32(~(-(382931935)), -(-(-948258457))), select(!(select(vec2<bool>(true, false), vec2<bool>(true, false), vec2<bool>(true, false))), !(!(vec2<bool>(false, true))), select(vec2<bool>(true, true), select(vec2<bool>(true, false), vec2<bool>(false, true), vec2<bool>(false, true)), func_2(vec3<u32>(1472518993u, 1726298634u, 1265906203u), 2427733236u, vec2<bool>(false, false)))), vec4<i32>(min(-(419139755), (649852032) ^ (64076662)), SAFE_PLUS_i32(max(-492524349, -2004291848), dot(vec4<i32>(1883511791, 1768122519, 1149599747, 1995509456), vec4<i32>(-1811337931, 678721919, -729821039, 1572650350))), SAFE_MINUS_i32(dot(vec3<i32>(1066662925, -2048124211, -1158378244), vec3<i32>(2041663729, 14912808, 1430825748)), -(1040664767)), -1816232574), SAFE_TIMES_i32(-1119941418, (dot(vec2<i32>(-532199109, -1817487469), vec2<i32>(1760828662, 752657207))) | (1859970733)), SAFE_PLUS_i32(func_3(select(vec3<bool>(true, false, true), vec3<bool>(true, false, false), true), Struct_1(vec3<i32>(357978136, -820767106, -1019065981), 831350817, vec2<bool>(true, true), vec4<i32>(2119538992, -757426996, 1581383502, -2017101551), 639415367, -1978510918, vec3<i32>(-1727787538, 2020282465, -1162100366)), clamp(vec3<i32>(-418074275, 1923654893, 310304064), vec3<i32>(1160643056, 148519818, 1617165098), vec3<i32>(982190885, -1222887415, 1042539872))), (-(-1306583944)) & ((1633531218) ^ (-854256166))), vec3<i32>(1713114456, ~((60255932) | (550725673)), min(max(2056826192, -1865455583), dot(vec4<i32>(-85794191, -923627444, -1044087546, -1228276307), vec4<i32>(1572148303, -360401567, -1031367338, -1405196813))))), max(SAFE_TIMES_vec2_i32(SAFE_DIVIDE_vec2_i32(min(vec2<i32>(-32784128, 476367333), vec2<i32>(968519296, 977090300)), SAFE_TIMES_vec2_i32(vec2<i32>(494674473, 1727981743), vec2<i32>(46016750, -2041034170))), min(vec2<i32>(441279617, 380047060), SAFE_TIMES_vec2_i32(vec2<i32>(-392182033, 1436983102), vec2<i32>(528211405, 243483955)))), (max(abs(vec2<i32>(-1256729835, -328849552)), vec2<i32>(-1195624632, -1767045169))) ^ (select((vec2<i32>(1049458202, 2009811791)) << (vec2<u32>(3078716239u, 29604321u)), min(vec2<i32>(-2091555853, 1256970321), vec2<i32>(-1189703959, -11286412)), select(vec2<bool>(false, false), vec2<bool>(false, true), true)))), dot(vec3<i32>(func_3(vec3<bool>(true, false, true), Struct_1(vec3<i32>(-12317777, 172544245, 2084783716), -1726417239, vec2<bool>(true, false), vec4<i32>(-186901098, 1486616241, -521321520, 1211244947), -64164920, 1096478321, vec3<i32>(1028206279, -421612833, -1875567805)), -(vec3<i32>(597359949, 1057635062, 946821941))), (-(1484924039)) & (1153869955), 2131939949), vec3<i32>(func_3(!(vec3<bool>(true, true, false)), Struct_1(vec3<i32>(-637977372, -1822301908, 1336450039), 1177481444, vec2<bool>(false, false), vec4<i32>(-1889531791, 523085566, -1355457223, -133197721), 1463387261, 88945991, vec3<i32>(1238183729, -363451051, 1002607530)), ~(vec3<i32>(-372049184, 725407466, -1224928970))), -(678445144), 320380360)), ~(SAFE_MOD_i32((2113290068) | (-277649441), (-(1442580519)) << (min(259462868u, 2005420293u)))), Struct_1((~(~(vec3<i32>(372522987, 1148250202, 761447911)))) >> (abs(SAFE_DIVIDE_vec3_u32(vec3<u32>(373901632u, 1005173277u, 1357754473u), vec3<u32>(4212803471u, 2742062746u, 1271762428u)))), -(SAFE_PLUS_i32(-1049144679, (-1923137605) | (1021939971))), !(select(!(vec2<bool>(false, false)), !(vec2<bool>(false, true)), !(vec2<bool>(true, false)))), (vec4<i32>(-(-1230686434), -(-1820198044), dot(vec4<i32>(764654496, -1324143637, 1001359927, 1126366884), vec4<i32>(-1964034747, 837850474, 1818895287, -869237314)), func_3(vec3<bool>(true, false, false), Struct_1(vec3<i32>(-1553232538, 1096800409, -1553622468), 1058573108, vec2<bool>(false, false), vec4<i32>(-790239775, 125474476, -954084301, 88500522), 1650320459, 1007490047, vec3<i32>(-1595005867, 464162791, -968937951)), vec3<i32>(-416724078, -870929467, -467053961)))) ^ (~(SAFE_PLUS_vec4_i32(vec4<i32>(1628090167, 1301466527, 1150900604, 1154004642), vec4<i32>(-1785185719, -837827016, -1695991243, 996366290)))), (abs(-(1316662673))) >> (clamp(~(846663140u), max(2309162297u, 1417732219u), 3207542549u)), (-((-448581134) & (852362646))) << (select(min(3070191663u, 16294937u), 114257805u, all(vec2<bool>(true, false)))), max(-(min(vec3<i32>(1588574136, 1916684593, 999736461), vec3<i32>(-963687615, -987518791, 1148876508))), vec3<i32>(func_3(vec3<bool>(false, false, false), Struct_1(vec3<i32>(1572020886, -23654044, 487394487), 1776560771, vec2<bool>(true, true), vec4<i32>(-854869446, -996578952, 1053930851, -385022938), 1579233650, -1571005017, vec3<i32>(-2078477849, -185594416, 1971349790)), vec3<i32>(1910693534, -1536699641, -1087151701)), max(466696431, -1019802695), (-126512282) ^ (-1452368568)))));
    let var_1 = any(var_0.a);
    var var_2 = false;
    var var_3 = Struct_1((vec3<i32>(1783510576, select(974533916, ~(var_0.h.e), var_1), -656954743)) & ((abs(var_0.h.g)) & (vec3<i32>(SAFE_PLUS_i32(var_0.f, 1432326038), var_0.f, var_0.d.g.x))), -(1793111250), select(!(vec2<bool>(any(vec2<bool>(var_0.a.x, var_1)), false)), select(!(!(vec2<bool>(var_0.d.c.x, var_0.h.c.x))), var_0.a.yz, any(select(var_0.a.zyw, var_0.a.zww, false))), (select(!(true), var_2, (var_1) != (true))) && (true)), SAFE_TIMES_vec4_i32(var_0.h.d, vec4<i32>(~(var_0.f), min(select(var_0.g, var_0.d.g.x, var_0.a.x), dot(vec2<i32>(var_0.e.x, var_0.g), var_0.d.a.zy)), -1205765529, (var_0.g) & (dot(var_0.h.d.xw, vec2<i32>(var_0.h.e, 1220462408))))), -(SAFE_DIVIDE_i32(var_0.f, min(abs(var_0.g), (-1387189260) << (1966863232u)))), var_0.d.f, vec3<i32>(~(var_0.f), var_0.d.f, dot(-(select(var_0.h.d, vec4<i32>(378771682, var_0.d.g.x, -913597590, var_0.d.g.x), var_0.a)), min(var_0.d.d, var_0.d.d))));
    loop {
        if ((LOOP_COUNTERS[11u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[11u] = (LOOP_COUNTERS[11u]) + (1u);
        var var_4 = var_3.d;
    }
    return var_2;
}

@stage(compute)
@workgroup_size(1)
fn main() {
    if (any(select(!(!(vec3<bool>(true, true, true))), vec3<bool>(true, all(!(vec2<bool>(true, false))), func_1(~(334595390u))), false))) {
        loop {
            if ((LOOP_COUNTERS[0u]) >= (1u)) {
                break;
            }
            LOOP_COUNTERS[0u] = (LOOP_COUNTERS[0u]) + (1u);
            loop {
                if ((LOOP_COUNTERS[1u]) >= (1u)) {
                    break;
                }
                LOOP_COUNTERS[1u] = (LOOP_COUNTERS[1u]) + (1u);
                if (!(!(!(select((-1419167497) != (1646048254), any(vec3<bool>(false, true, true)), all(vec3<bool>(true, false, true))))))) {
                    loop {
                        if ((LOOP_COUNTERS[2u]) >= (1u)) {
                            break;
                        }
                        LOOP_COUNTERS[2u] = (LOOP_COUNTERS[2u]) + (1u);
                        var var_0 = vec2<bool>(all(vec3<bool>(select((-1246801184) <= (111565600), !(false), all(vec4<bool>(false, false, true, false))), false, func_4(select(vec2<i32>(1143090651, 1552113778), vec2<i32>(1677292553, 1535432820), vec2<bool>(true, false)), Struct_2(vec4<bool>(true, true, false, false), vec2<u32>(4197474656u, 1272001443u), 558742639u, Struct_1(vec3<i32>(385691673, -295645229, 836726923), -1322851792, vec2<bool>(false, true), vec4<i32>(-1195462312, -744076917, 441063538, -620644270), -875797210, -1490422466, vec3<i32>(-1165406207, 423333239, -1767434751)), vec2<i32>(-292285259, 175354144), -166702957, 537760430, Struct_1(vec3<i32>(503613480, 1445479997, 467776592), 655552721, vec2<bool>(false, true), vec4<i32>(-480652479, 806239771, -253772595, 100923607), -1205504295, -569493198, vec3<i32>(1401401125, -1736732281, 279887084))), Struct_1(vec3<i32>(-549482048, -804308979, -1951819651), -906945175, vec2<bool>(true, true), vec4<i32>(1329525740, -486882246, -1015725130, 1647575512), -1196159429, -1330276597, vec3<i32>(1971850868, -1448164775, 49870107)), !(false)))), false);
                    }
                }
            }
        }
    }
    let var_0 = select(!(select(select(!(vec3<bool>(true, true, true)), select(vec3<bool>(true, false, false), vec3<bool>(false, false, true), vec3<bool>(false, true, true)), true), vec3<bool>((1063975811u) == (2398612559u), !(true), true), vec3<bool>((false) | (false), !(false), any(vec2<bool>(true, false))))), vec3<bool>(false, any(select(vec2<bool>(false, false), !(vec2<bool>(false, true)), vec2<bool>(true, true))), (!(true)) || ((!(false)) && ((2066236548u) < (2952975850u)))), select(vec3<bool>(!((true) && (true)), (select(-1574447374, -388181907, false)) >= (SAFE_PLUS_i32(-169775545, -1264528504)), (-935798072) >= (dot(vec2<i32>(1558923098, 1183614838), vec2<i32>(-767484752, 112102841)))), select(select(vec3<bool>(true, false, false), vec3<bool>(false, true, false), !(vec3<bool>(true, false, true))), !(!(vec3<bool>(false, true, false))), vec3<bool>(all(vec3<bool>(true, true, false)), func_1(2380138915u), (true) | (false))), any(select(select(vec3<bool>(false, true, false), vec3<bool>(true, true, false), false), !(vec3<bool>(true, true, true)), !(true)))));
    if (((var_0.x) | (!(!(false)))) | (func_1(1955399087u))) {
        var var_1 = Struct_2(vec4<bool>(var_0.x, (!(var_0.x)) || (true), !(!(!(false))), !(false)), vec2<u32>((SAFE_MINUS_u32(1098071292u, abs(1982517140u))) | (~(abs(756010133u))), SAFE_MOD_u32(2550987024u, ~(3805936004u))), ~(SAFE_DIVIDE_u32(dot((vec2<u32>(3173215055u, 1976312962u)) >> (vec2<u32>(2901245813u, 3392431388u)), ~(vec2<u32>(937817713u, 3222069798u))), 2896047323u)), Struct_1(abs(SAFE_DIVIDE_vec3_i32(abs(vec3<i32>(663055790, 474004807, 672308793)), max(vec3<i32>(1718504309, -1214190307, -903078847), vec3<i32>(-1550698746, 555118908, 226874040)))), -(dot((vec2<i32>(-150572812, 2005736864)) << (vec2<u32>(2518274940u, 2594111553u)), vec2<i32>(-1633253134, 412769724))), var_0.xx, SAFE_DIVIDE_vec4_i32(min(vec4<i32>(279820307, 1997506960, 57299917, 1738815475), vec4<i32>(-1555989162, -1903802778, 456018405, 133257898)), vec4<i32>((1925181740) ^ (923987728), (1448599315) & (556811961), -(1170858170), select(421515841, 998857838, true))), ~(dot(SAFE_MOD_vec4_i32(vec4<i32>(-271483620, 134658811, -1859539195, 892707258), vec4<i32>(-2112053610, -875736283, -161759421, -1094567510)), vec4<i32>(-682921340, -1608074554, 981857232, -832933829))), 1981377541, vec3<i32>(abs(dot(vec2<i32>(2062701069, 1852574108), vec2<i32>(50642806, -1837047799))), ~((885441193) >> (192833500u)), ((1361455846) ^ (979381709)) ^ (dot(vec4<i32>(1448765227, 2115256757, -1508670784, -2036859686), vec4<i32>(1783125960, -1717072612, 327009495, 288790067))))), select(select(~(vec2<i32>(1363567943, -113200363)), select(SAFE_MOD_vec2_i32(vec2<i32>(-304981443, 462213828), vec2<i32>(324517046, -2040907659)), -(vec2<i32>(-148757921, 580124169)), false), !(!(vec2<bool>(var_0.x, true)))), ~(min(vec2<i32>(1030042790, -475012210), SAFE_TIMES_vec2_i32(vec2<i32>(-1383730856, -1866665431), vec2<i32>(47590345, 1901473088)))), (SAFE_PLUS_u32((191492241u) << (3482105544u), ~(4165753822u))) == (~(SAFE_TIMES_u32(263978039u, 2987572004u)))), -(112048465), min(-2026539149, -((SAFE_DIVIDE_i32(1382797319, -1689820288)) ^ (dot(vec3<i32>(-82663071, 36281779, -2110941116), vec3<i32>(-104916745, -61716476, -287272675))))), Struct_1(-(max(vec3<i32>(-569817542, 1830097422, 1090310223), (vec3<i32>(526859250, -1474217575, -1779449008)) >> (vec3<u32>(1394100362u, 3463745772u, 1417441396u)))), SAFE_TIMES_i32(-(-2136989064), ~(~(121963183))), select(select(select(var_0.zx, vec2<bool>(var_0.x, false), var_0.zz), !(var_0.zx), select(var_0.xx, vec2<bool>(var_0.x, true), var_0.xy)), vec2<bool>(!(false), true), var_0.zy), vec4<i32>(SAFE_MINUS_i32((1008996737) | (-1729498054), -(-2003436000)), abs(-(2065388011)), ~(-1789955652), select(SAFE_PLUS_i32(911194173, 150481453), -168094810, var_0.x)), -1249626117, (-(-(-1890258157))) | ((~(-245754917)) ^ (min(-2077574010, 960254591))), ~(-(~(vec3<i32>(838168642, -293878115, -2065513764))))));
    }
    if (var_0.x) {
        if (func_1(~(dot(vec2<u32>(~(3853907223u), 1180030704u), (~(vec2<u32>(1602901638u, 4217180008u))) ^ (vec2<u32>(3345877541u, 1062759538u)))))) {
            var var_1 = Struct_1((vec3<i32>(func_3(select(var_0, var_0, var_0.x), Struct_1(vec3<i32>(686535406, -1792537207, -494071682), 1407347655, vec2<bool>(false, var_0.x), vec4<i32>(-2132050944, -719503371, 696679391, 1016595763), 1160270170, 910049308, vec3<i32>(-1619376463, 1267747490, -412671372)), (vec3<i32>(1218366190, 1895466662, -494722942)) << (vec3<u32>(2202473786u, 1010634418u, 1158864219u))), ~(1135280450), -(max(-2116590694, -296529090)))) ^ (min(vec3<i32>(~(1075639854), abs(-953440449), 1803509170), vec3<i32>(SAFE_MOD_i32(1331748442, 1748891456), select(1812533245, 1575834985, var_0.x), ~(-1341947301)))), select((~(-1595717828)) ^ (-(992128431)), -(-(107032769)), all(vec2<bool>(func_2(vec3<u32>(1324112867u, 3268277799u, 989676054u), 322857904u, var_0.xx), var_0.x))), var_0.xx, (vec4<i32>((SAFE_DIVIDE_i32(-613568997, -1602091568)) << (dot(vec4<u32>(567968935u, 2910914885u, 3767178066u, 427400221u), vec4<u32>(615867293u, 1801418622u, 3034967792u, 3783624197u))), max(dot(vec3<i32>(1586581305, -2057580390, 1757663532), vec3<i32>(1105560639, 1048016805, 657622590)), 190039874), ~(SAFE_MOD_i32(990890254, -1246936720)), max(~(302415514), SAFE_PLUS_i32(172777379, 1093149138)))) | ((vec4<i32>(-1144044038, (512667041) & (-2129265151), -1605409430, SAFE_DIVIDE_i32(-1571334226, 1200869038))) ^ (((vec4<i32>(-2139261947, 1279459224, -944705375, 1535690924)) | (vec4<i32>(-940136643, -1355396365, 915928305, -178611216))) << (~(vec4<u32>(1043468004u, 3537806494u, 3657412809u, 1147969603u))))), 919793366, -2113695823, vec3<i32>(-483225519, SAFE_MINUS_i32(-276671587, 2000514442), -(dot(vec4<i32>(-857724580, 1550342717, 1280174904, 1417248044), vec4<i32>(151809081, -1629207576, -91818830, -93140263)))));
        }
    }
    loop {
        if ((LOOP_COUNTERS[3u]) >= (1u)) {
            break;
        }
        LOOP_COUNTERS[3u] = (LOOP_COUNTERS[3u]) + (1u);
        let var_1 = true;
    }
    var var_1 = Struct_2(select(select(!(vec4<bool>(false, var_0.x, false, false)), vec4<bool>((297017019u) < (4029541298u), func_1(751209839u), (var_0.x) && (var_0.x), any(var_0)), all(var_0.zy)), select(!(!(vec4<bool>(var_0.x, var_0.x, false, false))), !(!(vec4<bool>(false, var_0.x, false, var_0.x))), !(!(vec4<bool>(false, var_0.x, true, true)))), !(vec4<bool>(!(var_0.x), !(var_0.x), true, var_0.x))), ~(vec2<u32>(dot(abs(vec3<u32>(1307217041u, 2370777868u, 733025701u)), ~(vec3<u32>(4226397391u, 1415857312u, 1766043871u))), select(SAFE_DIVIDE_u32(3116285565u, 499101554u), SAFE_MINUS_u32(4273798812u, 2342143406u), !(var_0.x)))), ~(max(3175798461u, dot(select(vec3<u32>(1108904782u, 2989492519u, 1240788915u), vec3<u32>(3531390522u, 2192988438u, 2684612614u), false), SAFE_DIVIDE_vec3_u32(vec3<u32>(1981759143u, 2305613275u, 292679173u), vec3<u32>(833376222u, 616592477u, 3215096891u))))), Struct_1(vec3<i32>(abs(dot(vec2<i32>(-1773211395, -861356223), vec2<i32>(1113931965, 41319463))), (~(895058944)) & (clamp(1923283932, 27915301, 1299575448)), -236374733), min((~(-1274222551)) ^ (-(1866070019)), -146921708), select(select(select(vec2<bool>(var_0.x, false), vec2<bool>(false, var_0.x), vec2<bool>(false, var_0.x)), var_0.yx, !(var_0.x)), select(vec2<bool>(false, true), select(var_0.xx, vec2<bool>(true, true), var_0.x), select(vec2<bool>(var_0.x, false), var_0.zx, var_0.yx)), func_1(SAFE_MINUS_u32(3764926276u, 1769418442u))), min(abs(vec4<i32>(1575974277, -165364023, 584829291, 645506701)), max(clamp(vec4<i32>(-1338220156, -1790680871, 1392000839, -1409889010), vec4<i32>(-334548576, -1124083846, 911417798, -1495863178), vec4<i32>(37048930, 360681187, 2136240312, 337760765)), vec4<i32>(-1743327594, 333154261, 2014761091, 1055140730))), (-(-(917333162))) & (-499248619), SAFE_MOD_i32((min(-676735521, 1722098436)) >> (535827340u), -1468786288), SAFE_DIVIDE_vec3_i32(((vec3<i32>(1248864434, -261017996, 915084047)) >> (vec3<u32>(2707942546u, 636373812u, 1194976956u))) << (vec3<u32>(4128359794u, 280935808u, 1493179969u)), ~(select(vec3<i32>(-1077725079, -679074094, 660466819), vec3<i32>(2105229095, 1758317329, 1968552039), var_0)))), clamp(~(-(select(vec2<i32>(1619021434, 338324960), vec2<i32>(443659421, 322233825), var_0.zz))), vec2<i32>(2141923834, -1013646535), SAFE_PLUS_vec2_i32(max(~(vec2<i32>(-1047274217, 646593851)), SAFE_TIMES_vec2_i32(vec2<i32>(-2058697386, 1614099823), vec2<i32>(-314206771, 310127841))), -(SAFE_TIMES_vec2_i32(vec2<i32>(803091756, -1571729184), vec2<i32>(-4000437, 1880765587))))), (select(-(SAFE_MOD_i32(1140626449, -1035360515)), -(-(-1033346260)), (dot(vec4<u32>(3420890047u, 4209872849u, 1713775829u, 3452280344u), vec4<u32>(282469654u, 1834045846u, 2621761853u, 1772341922u))) >= (~(1571406657u)))) | (~((dot(vec2<i32>(754823177, 1689380024), vec2<i32>(256112057, 1013233019))) | (588828162))), (-1542905042) & (1951403406), Struct_1(abs(~(~(vec3<i32>(1185018261, -1165475194, -984741766)))), ~(-742752214), vec2<bool>((var_0.x) | (var_0.x), (~(2895425659u)) >= (dot(vec2<u32>(3892196417u, 1644818963u), vec2<u32>(2687944077u, 1576038217u)))), vec4<i32>(~(-515753262), clamp(SAFE_PLUS_i32(2014355169, 5234495), SAFE_TIMES_i32(814106820, -820733909), clamp(-869343741, -98803619, 1679772607)), SAFE_MINUS_i32(~(-2142645858), 463139428), ~(~(286312690))), 547534102, -(dot((vec3<i32>(-1309881491, 1202817236, 1846276804)) & (vec3<i32>(841139799, -637781219, 1628623002)), -(vec3<i32>(-926001159, 1469719640, -1231397558)))), ~(vec3<i32>((1443352857) << (513455766u), 805851545, dot(vec4<i32>(1428056150, -589119431, 221745600, -1151955351), vec4<i32>(1742275333, 1483618872, 1135120547, -2072784679))))));
    output.value = input.value;
    output.value = 1375278764u;
}

