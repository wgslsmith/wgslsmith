fn SAFE_TIMES_i32(a: i32, b: i32) -> i32 {
    if (a == -1 && b == INT_MIN || a == INT_MIN && b == -1) {
        return a;
    }

    if (a > INT_MAX / b || a < INT_MIN / b) {
        return a;
    }

    return a * b;
}
