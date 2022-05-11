fn SAFE_MOD_i32(a: i32, b: i32) -> i32 {
    var safe_a = select(a, -a, a < 0);
    var safe_b = select(b, -b, b < 0);
    return select(safe_a % safe_b, safe_a % 2, safe_b == 0);
}
