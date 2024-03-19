use ast::*;
use crate::eval_value::Value;

pub enum Builtin {
    Abs,
    Exp2,
    CountOneBits,
    ReverseBits,
    FirstLeadingBit,
    Min,
}

impl Builtin {
    pub fn convert(ident : String) -> Option<Builtin> {
        match ident.as_str() {
            "exp2" => Some(Builtin::Exp2),
            "abs" => Some(Builtin::Abs),
            "countOneBits" => Some(Builtin::CountOneBits),
            "reverseBits" => Some(Builtin::ReverseBits),
            "firstLeadingBit" => Some(Builtin::FirstLeadingBit),
            "min" => Some(Builtin::Min),
            _ => None,
            }
    }
}

pub fn evaluate_builtin(ident : &Builtin, args : Vec<Option<Value>>) -> Option<Value> {

    // evaluate based on number of arguments passed to builtin
    match ident {
        Builtin::Min => {
            
            let arg1 = args[0].clone().unwrap();
            let arg2 = args[1].clone().unwrap();

            evaluate_two_arg_builtin(ident, arg1, arg2)
        },
        _ => {
            let single_arg = args[0].clone().unwrap();
            evaluate_single_arg_builtin(ident, single_arg)
        }
    }
}

fn evaluate_single_arg_builtin(ident : &Builtin, arg : Value) -> Option<Value> {

    match arg {
        Value::Lit(val) => {evaluate(ident, val)},
        Value::Vector(val) => {
            let mut result = Vec::new();
            
            for v in val {
                let elem = evaluate_single_arg_builtin(ident, v);

                match elem {
                    Some(e) => result.push(e),
                    None => {return None;},
                }
            }

            Some(Value::Vector(result))
        },
    }

}

fn evaluate_two_arg_builtin(ident : &Builtin, arg1 : Value, arg2 : Value) -> Option<Value> {
        
    match (arg1, arg2) {
        (Value::Lit(val1), Value::Lit(val2)) => {evaluate_two_args(ident, val1, val2)},
        (Value::Vector(val1), Value::Vector(val2)) => {
            let mut result = Vec::new();

            for (x, y) in val1.iter().zip(val2.iter()) {
                let elem = evaluate_two_arg_builtin(ident, x.clone(), y.clone());
                
                match elem {
                    Some(e) => result.push(e),
                    None => {return None;},
                }
            }

            Some(Value::Vector(result))
        },
        _ => todo!(), // cannot have mixed types in implemented builtin fn evaluation

    }
}

fn evaluate(ident : &Builtin, val : Lit) -> Option<Value> {

    match ident {
        Builtin::Exp2 => exp2(val),
        Builtin::Abs => abs(val),
        Builtin::CountOneBits => count_one_bits(val),
        Builtin::ReverseBits => reverse_bits(val),
        Builtin::FirstLeadingBit => first_leading_bit(val),
        _ => todo!(),
    }

}

fn evaluate_two_args(ident : &Builtin, val1 : Lit, val2 : Lit) -> Option<Value> {

    match ident {
        Builtin::Min => min(val1, val2),
        _ => todo!(),
    }
}

fn count_one_bits(val : Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Value::from_u32(Some(v.count_ones())),
        Lit::U32(v) => Value::from_u32(Some(v.count_ones())),
        _ => {None},
    }
 
}

fn abs(val : Lit) -> Option<Value> {
    
    match val {
        Lit::I32(v) => Value::from_i32(Some(v.abs())),
        Lit::F32(v) => Value::from_f32(Some(v.abs())),

        // abs() is not implemented for u32 in Rust, 
        // but it is implemented in WGSL
        Lit::U32(v) => Value::from_u32(Some(v)),
        _ => {None},
    }
}

fn exp2(val : Lit) -> Option<Value> {

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

            return Value::from_f32(result);
        }
        _ => None,   
    }    
}

//TODO: remove duplication of float range check
fn in_float_range(f : f32) -> Option<f32> {
    if f.abs() <= 0.1_f32 || f.abs() >= (16777216_f32) {
        return None;
    }
    else {return Some(f);}
} 

fn min(val1 : Lit, val2 : Lit) -> Option<Value> {
    match (val1, val2) {
        (Lit::I32(v1), Lit::I32(v2)) => Some(v1.min(v2).into()),
        (Lit::U32(v1), Lit::U32(v2)) => Some(v1.min(v2).into()),
        (Lit::F32(v1), Lit::F32(v2)) => Some(v1.min(v2).into()),
        _ => None,
    }
}

fn reverse_bits(val : Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => Some(v.reverse_bits().into()),
        Lit::U32(v) => Some(v.reverse_bits().into()),
        _ => None,
    }
}

fn first_leading_bit(val : Lit) -> Option<Value> {
    match val {
        Lit::I32(v) => {
            if v == 0 || v == 1 {
                Some((-1_i32).into())
            }
            else {
                Some(v.leading_zeros().into())
            }
        },
        Lit::U32(v) => Some(v.leading_zeros().into()),
        _ => None,
    }
}


