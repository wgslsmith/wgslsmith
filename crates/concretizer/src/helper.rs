use crate::value::Value;
use ast::Lit;

// Division
pub(crate) fn is_zero(val: &Value) -> bool {
    match val {
        Value::Lit(Lit::I32(v)) => *v == 0,
        Value::Lit(Lit::U32(v)) => *v == 0,
        Value::Lit(Lit::F32(v)) => *v == 0.0,
        Value::Vector(vec) => vec.iter().any(is_zero),
        _ => false,
    }
}

// insertBits/extractBits
pub fn is_invalid_bits_call(ident: &str, vals: &[Option<Value>]) -> bool {
    match ident {
        "extractBits" => check_offset_count_overflow(&vals[1], &vals[2]),
        "insertBits" => check_offset_count_overflow(&vals[2], &vals[3]),
        _ => false,
    }
}

fn check_offset_count_overflow(offset_arg: &Option<Value>, count_arg: &Option<Value>) -> bool {
    if let (Some(Value::Lit(Lit::U32(offset))), Some(Value::Lit(Lit::U32(count)))) =
        (offset_arg, count_arg)
    {
        (*offset as u64) + (*count as u64) > 32
    } else {
        false
    }
}

// Clamp
pub fn is_invalid_clamp_bounds(low: &Value, high: &Value) -> bool {
    match (low, high) {
        (Value::Lit(l), Value::Lit(h)) => match (l, h) {
            (Lit::I32(lv), Lit::I32(hv)) => lv > hv,
            (Lit::U32(lv), Lit::U32(hv)) => lv > hv,
            (Lit::F32(lv), Lit::F32(hv)) => lv > hv,
            _ => false,
        },
        (Value::Vector(l_vec), Value::Vector(h_vec)) => {
            if l_vec.len() != h_vec.len() {
                return false;
            }
            l_vec
                .iter()
                .zip(h_vec.iter())
                .any(|(l, h)| is_invalid_clamp_bounds(l, h))
        }
        // mixed scalar and vector (broadcasting)
        (Value::Lit(_), Value::Vector(h_vec)) => {
            h_vec.iter().any(|h| is_invalid_clamp_bounds(low, h))
        }
        (Value::Vector(l_vec), Value::Lit(_)) => {
            l_vec.iter().any(|l| is_invalid_clamp_bounds(l, high))
        }
    }
}
