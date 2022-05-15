fn SAFE_MINUS_u32(a: u32, b: u32) -> u32 {
    return select(a - b, b - a, a < b);
}
