use ast::*;
use crate::eval_value::Value;

macro_rules! abs {
    ($val:expr) => {
        match $val {
            Lit::I32(v) => Value::from_i32(Some(v.abs())),
            Lit::F32(v) => Value::from_f32(Some(v.abs())),

            // abs() is not implemented for u32 in Rust, 
            // but it is implemented in WGSL
            Lit::U32(v) => Value::from_u32(Some(v)),
            _ => {None},
        }
    }
}

macro_rules! countOnes {
    ($val:expr) => {
        match $val {
            Lit::I32(v) => Value::from_u32(Some(v.count_ones())),
            Lit::U32(v) => Value::from_u32(Some(v.count_ones())),
            _ => {None},
        }
    }
}

pub enum Builtin {
    Abs,
    Exp2,
    CountOneBits,
    ReverseBits,
    Min,
}

impl Builtin {
    pub fn convert(ident : String) -> Option<Builtin> {
        match ident.as_str() {
            "exp2" => Some(Builtin::Exp2),
            "abs" => Some(Builtin::Abs),
            "countOneBits" => Some(Builtin::CountOneBits),
            "reverseBits" => Some(Builtin::ReverseBits),
            "min" => Some(Builtin::Min),
            _ => None,
            }
    }
}

pub fn evaluate_builtin(ident : &Builtin, args : Value) -> Option<Value> {

    match args {
        Value::Lit(val) => {evaluate(ident, val)},
        Value::Vector(val) => {
            let mut result = Vec::new();
            
            for v in val {
                let elem = evaluate_builtin(ident, v);

                    match elem {
                        Some(e) => result.push(e),
                        None => {return None;},
                    }
            }

            Some(Value::Vector(result))
        },
    }

}

fn evaluate(ident : &Builtin, val : Lit) -> Option<Value> {

    match ident {
        Builtin::Exp2 => exp2(val),
        Builtin::Abs => abs(val),
        Builtin::CountOneBits => count_one_bits(val),
        Builtin::ReverseBits => todo!(),
        Builtin::Min => todo!(),
    }

}

fn count_one_bits(val : Lit) -> Option<Value> {
    countOnes!(val)
}

fn abs(val : Lit) -> Option<Value> {
    abs!(val)
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


