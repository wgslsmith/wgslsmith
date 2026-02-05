use crate::concretizer::in_float_range;
use crate::value;
use ast::*;
use value::Value;

#[derive(Debug)]
pub enum Builtin {
    Abs,
    Clamp,
    All,
    Any,
    Dot,
    Exp,
    Exp2,
    CountLeadingZeros,
    CountTrailingZeros,
    CountOneBits,
    ExtractBits,
    InsertBits,
    ReverseBits,
    FirstLeadingBit,
    FirstTrailingBit,
    Min,
    Max,
    Select,
}

impl Builtin {
    pub fn convert(ident: String) -> Option<Builtin> {
        match ident.as_str() {
            "clamp" => Some(Builtin::Clamp),
            "exp" => Some(Builtin::Exp),
            "exp2" => Some(Builtin::Exp2),
            "all" => Some(Builtin::All),
            "any" => Some(Builtin::Any),
            "abs" => Some(Builtin::Abs),
            "countLeadingZeros" => Some(Builtin::CountLeadingZeros),
            "countTrailingZeros" => Some(Builtin::CountTrailingZeros),
            "countOneBits" => Some(Builtin::CountOneBits),
            "extractBits" => Some(Builtin::ExtractBits),
            "insertBits" => Some(Builtin::InsertBits),
            "reverseBits" => Some(Builtin::ReverseBits),
            "firstLeadingBit" => Some(Builtin::FirstLeadingBit),
            "firstTrailingBit" => Some(Builtin::FirstTrailingBit),
            "min" => Some(Builtin::Min),
            "max" => Some(Builtin::Max),
            "select" => Some(Builtin::Select),
            "dot" => Some(Builtin::Dot),
            _ => None,
        }
    }
}

pub fn evaluate_builtin(ident: &Builtin, args: Vec<Option<Value>>) -> Option<Value> {
    // evaluate based on number of arguments passed to builtin
    match ident {
        Builtin::InsertBits => {
            let arg1 = args[0].clone().unwrap();
            let arg2 = args[1].clone().unwrap();
            let arg3 = args[2].clone().unwrap();
            let arg4 = args[3].clone().unwrap();

            evaluate_four_arg_builtin(ident, arg1, arg2, arg3, arg4)
        }

        Builtin::Select | Builtin::ExtractBits | Builtin::Clamp => {
            let arg1 = args[0].clone().unwrap();
            let arg2 = args[1].clone().unwrap();
            let arg3 = args[2].clone().unwrap();

            evaluate_three_arg_builtin(ident, arg1, arg2, arg3)
        }

        Builtin::Min | Builtin::Max => {
            let arg1 = args[0].clone().unwrap();
            let arg2 = args[1].clone().unwrap();

            evaluate_two_arg_builtin(ident, arg1, arg2)
        }

        Builtin::Dot => {
            let arg1 = args[0].clone().unwrap();
            let arg2 = args[1].clone().unwrap();

            evaluate_dot(arg1, arg2)
        }

        // these are reductions (Vector -> Scalar), so we can't use evaluate_single_arg_builtin
        Builtin::All | Builtin::Any => {
            let arg = args[0].clone().unwrap();
            evaluate_bool_reduction(ident, arg)
        }

        _ => {
            let single_arg = args[0].clone().unwrap();
            evaluate_single_arg_builtin(ident, single_arg)
        }
    }
}

fn evaluate_single_arg_builtin(ident: &Builtin, arg: Value) -> Option<Value> {
    match arg {
        Value::Lit(val) => evaluate(ident, val),
        Value::Vector(val) => {
            let mut result = Vec::new();

            for v in val {
                let elem = evaluate_single_arg_builtin(ident, v);

                match elem {
                    Some(e) => result.push(e),
                    None => {
                        return None;
                    }
                }
            }

            Some(Value::Vector(result))
        }
    }
}

fn evaluate_two_arg_builtin(ident: &Builtin, arg1: Value, arg2: Value) -> Option<Value> {
    match (arg1, arg2) {
        (Value::Lit(val1), Value::Lit(val2)) => evaluate_two_args(ident, val1, val2),
        (Value::Vector(val1), Value::Vector(val2)) => {
            let mut result = Vec::new();

            for (x, y) in val1.iter().zip(val2.iter()) {
                let elem = evaluate_two_arg_builtin(ident, x.clone(), y.clone());

                match elem {
                    Some(e) => result.push(e),
                    None => {
                        return None;
                    }
                }
            }

            Some(Value::Vector(result))
        }
        _ => todo!(),
    }
}

fn evaluate_three_arg_builtin(
    ident: &Builtin,
    arg1: Value,
    arg2: Value,
    arg3: Value,
) -> Option<Value> {
    match (arg1, arg2, arg3) {
        (Value::Lit(val1), Value::Lit(val2), Value::Lit(val3)) => {
            evaluate_three_args(ident, val1, val2, val3)
        }
        (Value::Vector(val1), Value::Vector(val2), Value::Vector(val3)) => {
            let mut result = Vec::new();

            for ((x, y), z) in val1.iter().zip(val2.iter()).zip(val3.iter()) {
                let elem = evaluate_three_arg_builtin(ident, x.clone(), y.clone(), z.clone());

                match elem {
                    Some(e) => result.push(e),
                    None => {
                        return None;
                    }
                }
            }

            Some(Value::Vector(result))
        }
        _ => None,
    }
}

fn evaluate_four_arg_builtin(
    ident: &Builtin,
    arg1: Value,
    arg2: Value,
    arg3: Value,
    arg4: Value,
) -> Option<Value> {
    match (arg1, arg2, arg3, arg4) {
        (Value::Lit(val1), Value::Lit(val2), Value::Lit(val3), Value::Lit(val4)) => {
            evaluate_four_args(ident, val1, val2, val3, val4)
        }
        (Value::Vector(val1), Value::Vector(val2), Value::Vector(val3), Value::Vector(val4)) => {
            let mut result = Vec::new();
            for (((x, y), z), w) in val1
                .iter()
                .zip(val2.iter())
                .zip(val3.iter())
                .zip(val4.iter())
            {
                let elem =
                    evaluate_four_arg_builtin(ident, x.clone(), y.clone(), z.clone(), w.clone());

                match elem {
                    Some(e) => result.push(e),
                    None => return None,
                }
            }
            Some(Value::Vector(result))
        }
        _ => None,
    }
}

fn evaluate(ident: &Builtin, val: Lit) -> Option<Value> {
    match ident {
        Builtin::Exp => exp(val),
        Builtin::Exp2 => exp2(val),
        Builtin::Abs => abs(val),
        Builtin::CountOneBits => count_one_bits(val),
        Builtin::CountLeadingZeros => count_leading_zeros(val),
        Builtin::CountTrailingZeros => count_trailing_zeros(val),
        Builtin::ReverseBits => reverse_bits(val),
        Builtin::FirstLeadingBit => first_leading_bit(val),
        Builtin::FirstTrailingBit => first_trailing_bit(val),
        _ => todo!(),
    }
}

fn evaluate_two_args(ident: &Builtin, val1: Lit, val2: Lit) -> Option<Value> {
    match ident {
        Builtin::Min => min(val1, val2),
        Builtin::Max => max(val1, val2),
        _ => todo!(),
    }
}

fn evaluate_three_args(ident: &Builtin, val1: Lit, val2: Lit, val3: Lit) -> Option<Value> {
    match ident {
        Builtin::Select => select(val1, val2, val3),
        Builtin::ExtractBits => extract_bits(val1, val2, val3),
        Builtin::Clamp => clamp(val1, val2, val3),
        _ => todo!(),
    }
}

fn evaluate_four_args(
    ident: &Builtin,
    val1: Lit,
    val2: Lit,
    val3: Lit,
    val4: Lit,
) -> Option<Value> {
    match ident {
        Builtin::InsertBits => insert_bits(val1, val2, val3, val4),
        _ => todo!(),
    }
}

fn evaluate_bool_reduction(ident: &Builtin, arg: Value) -> Option<Value> {
    match arg {
        Value::Vector(vec) => {
            let mut bool_values = Vec::with_capacity(vec.len());

            for v in vec {
                match v {
                    Value::Lit(Lit::Bool(b)) => bool_values.push(b),
                    _ => return None,
                }
            }

            let result = match ident {
                Builtin::Any => bool_values.into_iter().any(|b| b),
                Builtin::All => bool_values.into_iter().all(|b| b),
                _ => return None,
            };

            Some(result.into())
        }
        Value::Lit(Lit::Bool(b)) => Some(b.into()),
        _ => None,
    }
}

fn count_one_bits(val: Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Value::from_i32(i32::try_from(v.count_ones()).ok()),
        Lit::U32(v) => Value::from_u32(Some(v.count_ones())),
        _ => None,
    }
}

fn count_leading_zeros(val: Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Value::from_i32(Some(v.leading_zeros() as i32)),
        Lit::U32(v) => Value::from_u32(Some(v.leading_zeros())),
        _ => None,
    }
}

fn count_trailing_zeros(val: Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Value::from_i32(Some(v.trailing_zeros() as i32)),
        Lit::U32(v) => Value::from_u32(Some(v.trailing_zeros())),
        _ => None,
    }
}

fn extract_bits(val: Lit, offset_arg: Lit, count_arg: Lit) -> Option<Value> {
    // offset and count must be u32
    let offset = match offset_arg {
        Lit::U32(u) => u,
        _ => return None,
    };
    let count = match count_arg {
        Lit::U32(u) => u,
        _ => return None,
    };

    if (offset as u64) + (count as u64) > 32 {
        return None;
    }

    // If count is 0, result is 0
    if count == 0 {
        match val {
            Lit::I32(_) => return Some(0.into()),
            Lit::U32(_) => return Some(0.into()),
            _ => return None,
        }
    }

    match val {
        Lit::I32(v) => {
            // Signed extract: Sign-extend from the (count-1)th bit of the result.
            // Shift left to clear upper bits, arithmetic shift right to restore position and sign-extend.
            let shift_left = 32 - (offset + count);
            let shift_right = 32 - count;

            let result = (v << shift_left) >> shift_right;
            Some(result.into())
        }
        Lit::U32(v) => {
            // Unsigned extract: Zero-extend.
            // Shift right to move to LSB, mask out upper bits.
            let shifted = v >> offset;
            // Handle count=32 carefully to avoid overflow in mask generation (1 << 32)
            let mask = if count == 32 {
                u32::MAX
            } else {
                (1 << count) - 1
            };

            Some((shifted & mask).into())
        }
        _ => None,
    }
}

fn clamp(e: Lit, low: Lit, high: Lit) -> Option<Value> {
    match (e, low, high) {
        (Lit::I32(e_val), Lit::I32(low_val), Lit::I32(high_val)) => {
            if low_val > high_val {
                return None;
            }
            Some(e_val.clamp(low_val, high_val).into())
        }
        (Lit::U32(e_val), Lit::U32(low_val), Lit::U32(high_val)) => {
            if low_val > high_val {
                return None;
            }
            Some(e_val.clamp(low_val, high_val).into())
        }
        (Lit::F32(e_val), Lit::F32(low_val), Lit::F32(high_val)) => {
            if low_val > high_val {
                return None;
            }
            Some(e_val.clamp(low_val, high_val).into())
        }
        _ => None,
    }
}

fn insert_bits(e_arg: Lit, newbits_arg: Lit, offset_arg: Lit, count_arg: Lit) -> Option<Value> {
    let offset = match offset_arg {
        Lit::U32(u) => u,
        _ => return None,
    };
    let count = match count_arg {
        Lit::U32(u) => u,
        _ => return None,
    };

    if (offset as u64) + (count as u64) > 32 {
        return None;
    }

    if count == 0 {
        match e_arg {
            Lit::I32(v) => return Some(v.into()),
            Lit::U32(v) => return Some(v.into()),
            _ => return None,
        }
    }

    // Helper to perform the bitwise logic on raw u32 bits
    let calc_insert = |e_raw: u32, new_raw: u32| -> u32 {
        let mask_width = if count == 32 {
            u32::MAX
        } else {
            (1 << count) - 1
        };
        let mask = mask_width << offset;

        (e_raw & !mask) | ((new_raw & mask_width) << offset)
    };

    match (e_arg, newbits_arg) {
        (Lit::I32(e), Lit::I32(newbits)) => {
            let result_raw = calc_insert(e as u32, newbits as u32);
            Some((result_raw as i32).into())
        }
        (Lit::U32(e), Lit::U32(newbits)) => {
            let result = calc_insert(e, newbits);
            Some(result.into())
        }
        _ => None,
    }
}

fn abs(val: Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Value::from_i32(Some(v.wrapping_abs())),
        Lit::F32(v) => Value::from_f32(Some(v.abs())),
        Lit::U32(v) => Value::from_u32(Some(v)),
        _ => None,
    }
}

fn exp(val: Lit) -> Option<Value> {
    match val {
        Lit::F32(v) => {
            // The maximum representable f32 is approx 3.4028e38.
            // ln(3.4028e38) approx 88.72.
            // If v > 88.72, exp(v) overflows f32.
            if v > 88.72_f32 {
                return None;
            }

            let result = in_float_range(v.exp());
            Value::from_f32(result)
        }
        _ => None,
    }
}

fn exp2(val: Lit) -> Option<Value> {
    match val {
        Lit::F32(v) => {
            // approximation - the maximum representable f32
            // is (2 - 2^-23)*2^127. so, conservatively if
            // v > 127 then exp2(v) is not representable as
            // a concrete f32. so replace node if v > 127
            if v > 127.0_f32 {
                return None;
            }

            // otherwise, if v < 127 we need to check that
            // the result is precisely representable. another
            // approximation is used to restrict f32 value to
            // the precicely representable range (as in
            // in_float_range(f32)
            let a = 2.0_f32;

            let result = in_float_range(a.powf(v));

            Value::from_f32(result)
        }
        _ => None,
    }
}

fn min(val1: Lit, val2: Lit) -> Option<Value> {
    match (val1, val2) {
        (Lit::I32(v1), Lit::I32(v2)) => Some(v1.min(v2).into()),
        (Lit::U32(v1), Lit::U32(v2)) => Some(v1.min(v2).into()),
        (Lit::F32(v1), Lit::F32(v2)) => Some(v1.min(v2).into()),
        _ => None,
    }
}

fn max(val1: Lit, val2: Lit) -> Option<Value> {
    match (val1, val2) {
        (Lit::I32(v1), Lit::I32(v2)) => Some(v1.max(v2).into()),
        (Lit::U32(v1), Lit::U32(v2)) => Some(v1.max(v2).into()),
        (Lit::F32(v1), Lit::F32(v2)) => Some(v1.max(v2).into()),
        _ => None,
    }
}

fn select(val1: Lit, val2: Lit, val3: Lit) -> Option<Value> {
    let cond = match val3 {
        Lit::Bool(b) => b,
        _ => return None,
    };

    match (val1, val2) {
        (Lit::I32(v1), Lit::I32(v2)) => Some(if cond { v2 } else { v1 }.into()),
        (Lit::U32(v1), Lit::U32(v2)) => Some(if cond { v2 } else { v1 }.into()),
        (Lit::F32(v1), Lit::F32(v2)) => Some(if cond { v2 } else { v1 }.into()),
        // (Lit::Bool(v1), Lit::Bool(v2)) => Some(if cond { v2 } else { v1 }.into()),
        _ => None,
    }
}

fn reverse_bits(val: Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Some(v.reverse_bits().into()),
        Lit::U32(v) => Some(v.reverse_bits().into()),
        _ => None,
    }
}

fn first_leading_bit(val: Lit) -> Option<Value> {
    match val {
        // signed: returns index of the first bit that differs from the sign bit
        // if value is 0 or -1, returns -1
        Lit::I32(v) => {
            // If v is negative, invert it.
            // This maps -1 (...111) to 0 (...000), making the logic identical to positive numbers.
            // i.e., finding the highest '1' of `!v` is finding the highest '0' in `v`.
            let effective_v = if v < 0 { !v } else { v };

            if effective_v == 0 {
                return Some((-1_i32).into());
            }

            let leading_zeros = effective_v.leading_zeros();
            let index = 31 - leading_zeros;
            Some((index as i32).into())
        }
        // unsigned: returns index of first set bit
        // if value is 0, returns -1 (u32::MAX)
        Lit::U32(v) => {
            if v == 0 {
                return Some(u32::MAX.into());
            }

            let index = 31 - v.leading_zeros();
            Some(index.into())
        }
        _ => None,
    }
}

fn first_trailing_bit(val: Lit) -> Option<Value> {
    match val {
        // signed: returns index of the least significant set bit
        // if value is 0, returns -1
        // Note: WGSL treats the bits as raw bits, so sign doesn't change the logic.
        Lit::I32(v) => {
            if v == 0 {
                return Some((-1_i32).into());
            }

            let index = v.trailing_zeros();
            Some((index as i32).into())
        }

        // unsigned: returns index of the least significant set bit
        // if value is 0, returns -1 (u32::MAX)
        Lit::U32(v) => {
            if v == 0 {
                return Some(u32::MAX.into());
            }

            let index = v.trailing_zeros();
            Some(index.into())
        }
        _ => None,
    }
}

fn evaluate_dot(arg1: Value, arg2: Value) -> Option<Value> {
    // Naga uses checked_mul, tint uses wrapping_mul.
    // Could change this to wrapping_mul if this gets answered https://github.com/gfx-rs/wgpu/issues/8900#issuecomment-3850412118
    match (arg1, arg2) {
        (Value::Vector(v1), Value::Vector(v2)) => {
            if v1.len() != v2.len() || v1.is_empty() {
                return None;
            }

            match (&v1[0], &v2[0]) {
                (Value::Lit(Lit::I32(_)), Value::Lit(Lit::I32(_))) => {
                    let mut sum = 0i32;
                    for (x, y) in v1.iter().zip(v2.iter()) {
                        if let (Value::Lit(Lit::I32(xv)), Value::Lit(Lit::I32(yv))) = (x, y) {
                            let product = xv.checked_mul(*yv)?;
                            sum = sum.checked_add(product)?;
                        } else {
                            return None;
                        }
                    }
                    Some(sum.into())
                }
                (Value::Lit(Lit::U32(_)), Value::Lit(Lit::U32(_))) => {
                    let mut sum = 0u32;
                    for (x, y) in v1.iter().zip(v2.iter()) {
                        if let (Value::Lit(Lit::U32(xv)), Value::Lit(Lit::U32(yv))) = (x, y) {
                            let product = xv.checked_mul(*yv)?;
                            sum = sum.checked_add(product)?;
                        } else {
                            return None;
                        }
                    }
                    Some(sum.into())
                }
                (Value::Lit(Lit::F32(_)), Value::Lit(Lit::F32(_))) => {
                    let mut sum = 0.0f32;
                    for (x, y) in v1.iter().zip(v2.iter()) {
                        if let (Value::Lit(Lit::F32(xv)), Value::Lit(Lit::F32(yv))) = (x, y) {
                            let product = xv * yv;
                            in_float_range(product)?;
                            sum += product;
                            in_float_range(sum)?;
                        } else {
                            return None;
                        }
                    }
                    Some(sum.into())
                }
                _ => None,
            }
        }
        _ => None,
    }
}
