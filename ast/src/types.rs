use std::fmt::{self, Display};
use std::rc::Rc;

use crate::StructDecl;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    I32,
    U32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DataType {
    Scalar(ScalarType),
    Vector(u8, ScalarType),
    Array(Rc<DataType>),
    User(Rc<StructDecl>),
}

impl DataType {
    pub fn map(&self, scalar: ScalarType) -> DataType {
        match self {
            DataType::Scalar(_) => DataType::Scalar(scalar),
            DataType::Vector(n, _) => DataType::Vector(*n, scalar),
            DataType::Array(_) => unimplemented!(),
            DataType::User(_) => unimplemented!(),
        }
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ScalarType::Bool => "bool",
            ScalarType::I32 => "i32",
            ScalarType::U32 => "u32",
        })
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Scalar(t) => write!(f, "{}", t),
            DataType::Vector(n, t) => write!(f, "vec{}<{}>", n, t),
            DataType::Array(inner) => write!(f, "array<{}>", inner),
            DataType::User(decl) => write!(f, "{}", decl.name),
        }
    }
}
