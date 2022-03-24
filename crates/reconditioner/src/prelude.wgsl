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
