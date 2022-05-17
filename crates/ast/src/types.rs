use std::fmt::{self, Display};
use std::rc::Rc;

use derive_more::Display;

use crate::StructDecl;

#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Eq)]
pub enum ScalarType {
    #[display(fmt = "bool")]
    Bool,
    #[display(fmt = "i32")]
    I32,
    #[display(fmt = "u32")]
    U32,
    #[display(fmt = "f32")]
    F32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DataType {
    Scalar(ScalarType),
    Vector(u8, ScalarType),
    Array(Rc<DataType>, Option<u32>),
    Struct(Rc<StructDecl>),
    Ptr(Rc<DataType>),
}

impl DataType {
    pub fn array(element_type: impl Into<DataType>, size: impl Into<Option<u32>>) -> DataType {
        DataType::Array(Rc::new(element_type.into()), size.into())
    }

    pub fn map(&self, scalar: ScalarType) -> DataType {
        match self {
            DataType::Scalar(_) => DataType::Scalar(scalar),
            DataType::Vector(n, _) => DataType::Vector(*n, scalar),
            DataType::Array(..) => unimplemented!(),
            DataType::Struct(_) => unimplemented!(),
            DataType::Ptr(..) => unimplemented!(),
        }
    }

    pub fn as_scalar(&self) -> Option<ScalarType> {
        match self {
            DataType::Scalar(ty) => Some(*ty),
            DataType::Vector(_, ty) => Some(*ty),
            DataType::Array(..) => None,
            DataType::Struct(_) => None,
            DataType::Ptr(..) => None,
        }
    }

    pub fn dereference(&self) -> Option<&DataType> {
        if let DataType::Ptr(inner, ..) = self {
            Some(inner)
        } else {
            None
        }
    }

    /// Returns `true` if the data type is [`Scalar`].
    ///
    /// [`Scalar`]: DataType::Scalar
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar(..))
    }

    /// Returns `true` if the data type is [`Vector`].
    ///
    /// [`Vector`]: DataType::Vector
    #[must_use]
    pub fn is_vector(&self) -> bool {
        matches!(self, Self::Vector(..))
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Scalar(t) => write!(f, "{}", t),
            DataType::Vector(n, t) => write!(f, "vec{}<{}>", n, t),
            DataType::Array(inner, n) => {
                write!(f, "array<{inner}")?;
                if let Some(n) = n {
                    write!(f, ", {n}")?;
                }
                write!(f, ">")
            }
            DataType::Struct(decl) => write!(f, "{}", decl.name),
            DataType::Ptr(_) => todo!(),
        }
    }
}

impl From<ScalarType> for DataType {
    fn from(scalar: ScalarType) -> Self {
        DataType::Scalar(scalar)
    }
}

impl From<&ScalarType> for DataType {
    fn from(scalar: &ScalarType) -> Self {
        DataType::Scalar(*scalar)
    }
}
