fn SAFE_MOD_i32(a: i32, b: i32) -> i32 {
    if (b == 0) {
        return a % 2;
    } else {
        return a % b;
    }
}
