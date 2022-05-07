fn SAFE_PLUS_i32(a: i32, b: i32) -> i32 {
    if (b > 0 && a > INT_MAX - b || b < 0 && a < INT_MIN - b) {
        return a;
    } else {
        return a + b;
    }
}
