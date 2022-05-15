fn SAFE_TIMES_u32(a: u32, b: u32) -> u32 {
    return select(a * b, ((a - (UINT_MAX / b)) % (UINT_MAX / b)) * b, a > UINT_MAX / b);
}
