use ast::*;

#[derive(Clone, Debug)]
pub enum Value {
    Lit(Lit),
    Vector(Vec<Value>),
}
impl From<i32> for Value {
    fn from(val: i32) -> Self {
        Value::Lit(Lit::I32(val))
    }
}
impl From<u32> for Value {
    fn from(val: u32) -> Self {
        Value::Lit(Lit::U32(val))
    }
}
impl From<f32> for Value {
    fn from(val: f32) -> Self {
        Value::Lit(Lit::F32(val))
    }
}
impl Value {
    pub fn from_i32(val: Option<i32>) -> Option<Value> {
        match val {
            Some(i) => Some(Value::Lit(Lit::I32(i))),
            None => None,
        }
    }
    pub fn from_u32(val: Option<u32>) -> Option<Value> {
        match val {
            Some(i) => Some(Value::Lit(Lit::U32(i))),
            None => None,
        }
    }
    pub fn from_f32(val: Option<f32>) -> Option<Value> {
        match val {
            Some(i) => Some(Value::Lit(Lit::F32(i))),
            None => None,
        }
    }
    pub fn from_bool(val: Option<bool>) -> Option<Value> {
        match val {
            Some(i) => Some(Value::Lit(Lit::Bool(i))),
            None => None,
        }
    }
}
