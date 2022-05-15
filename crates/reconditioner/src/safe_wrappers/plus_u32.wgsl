fn SAFE_PLUS_u32(a: u32, b: u32) -> u32 {
    return select(a + b, a - (UINT_MAX - b), a > UINT_MAX - b);
}
