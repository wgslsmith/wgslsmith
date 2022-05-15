fn SAFE_MOD_u32(a: u32, b: u32) -> u32 {
    return select(a % b, a % 2u, b == 0u);
}
