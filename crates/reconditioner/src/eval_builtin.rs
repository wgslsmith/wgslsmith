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
}

impl Builtin {
    pub fn convert(ident : String) -> Option<Builtin> {
        match ident.as_str() {
            "exp2" => Some(Builtin::Exp2),
            "abs" => Some(Builtin::Abs),
            "countOneBits" => Some(Builtin::CountOneBits),
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
        Builtin::CountOneBits => countOneBits(val),
    }

}

fn countOneBits(val : Lit) -> Option<Value> {
    countOnes!(val)
}

fn abs(val : Lit) -> Option<Value> {
    todo!()
}

fn exp2(val : Lit) -> Option<Value> {
    todo!()
}

