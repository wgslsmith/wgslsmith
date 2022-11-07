use std::fmt::{self, Display};
use std::rc::Rc;

use derive_more::Display;

use crate::{AccessMode, StorageClass, StructDecl};

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
    #[display(fmt = "atomic<u32>")]
    AU32,
    #[display(fmt = "atomic<i32>")]
    AI32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MemoryViewType {
    pub inner: Rc<DataType>,
    pub storage_class: StorageClass,
    pub access_mode: AccessMode,
}

impl MemoryViewType {
    pub fn new(inner: impl Into<DataType>, storage_class: StorageClass) -> MemoryViewType {
        MemoryViewType {
            inner: Rc::new(inner.into()),
            storage_class,
            access_mode: storage_class.default_access_mode(),
        }
    }

    pub fn clone_with_type(&self, inner: impl Into<DataType>) -> MemoryViewType {
        MemoryViewType {
            inner: Rc::new(inner.into()),
            ..*self
        }
    }
}

impl Display for MemoryViewType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.storage_class, self.inner)?;
        if self.access_mode != self.storage_class.default_access_mode() {
            write!(f, ", {}", self.access_mode)?;
        }
        Ok(())
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum DataType {
    Scalar(ScalarType),
    Vector(u8, ScalarType),
    Array(Rc<DataType>, Option<u32>),
    Struct(Rc<StructDecl>),
    Ptr(MemoryViewType),
    Ref(MemoryViewType),
}

impl DataType {
    pub fn array(element_type: impl Into<DataType>, size: impl Into<Option<u32>>) -> DataType {
        DataType::Array(Rc::new(element_type.into()), size.into())
    }

    pub fn map(&self, scalar: ScalarType) -> DataType {
        match self {
            DataType::Scalar(_) => DataType::Scalar(scalar),
            DataType::Vector(n, _) => DataType::Vector(*n, scalar),
            _ => unimplemented!(),
        }
    }

    pub fn as_scalar(&self) -> Option<ScalarType> {
        match self {
            DataType::Scalar(ty) => Some(*ty),
            DataType::Vector(_, ty) => Some(*ty),
            DataType::Ref(view) => view.inner.as_scalar(),
            _ => None,
        }
    }

    pub fn as_memory_view(&self) -> Option<&MemoryViewType> {
        if let DataType::Ptr(view) | DataType::Ref(view) = self {
            Some(view)
        } else {
            None
        }
    }

    /// Returns the referenced type if `self` is a reference, otherwise returns `self`.
    ///
    /// Note that this will not dereference a pointer type (only a reference).
    pub fn dereference(&self) -> &DataType {
        if let DataType::Ref(view) = self {
            view.inner.as_ref()
        } else {
            self
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

    /// Returns `true` if the data type is a scalar or vector of integers.
    pub fn is_integer(&self) -> bool {
        matches!(self.as_scalar(), Some(ScalarType::I32 | ScalarType::U32))
    }
}

impl fmt::Debug for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(arg0) => f.debug_tuple("Scalar").field(arg0).finish(),
            Self::Vector(arg0, arg1) => f.debug_tuple("Vector").field(arg0).field(arg1).finish(),
            Self::Array(arg0, arg1) => f.debug_tuple("Array").field(arg0).field(arg1).finish(),
            Self::Struct(arg0) => f.debug_tuple("Struct").field(&arg0.name).finish(),
            Self::Ptr(arg0) => f.debug_tuple("Ptr").field(arg0).finish(),
            Self::Ref(arg0) => f.debug_tuple("Ref").field(arg0).finish(),
        }
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
            DataType::Ptr(view) => write!(f, "ptr<{view}>"),
            DataType::Ref(view) => write!(f, "ref<{view}>"),
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
