use std::fmt::{self, Display};
use std::iter;

use once_cell::sync::OnceCell;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::HashTrieSetSync;

use crate::define_type;

#[derive(Clone, PartialEq, Eq)]
pub struct TypeConstraints(HashTrieSetSync<DataType>);

impl fmt::Debug for TypeConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    I32,
    U32,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DataType {
    Scalar(ScalarType),
    Vector(u8, ScalarType),
}

define_type!(scalar: Bool);
define_type!(scalar: I32);
define_type!(scalar: U32);

define_type!(
    vec:
        (2, Bool),
        (2, I32),
        (2, U32),
        (3, Bool),
        (3, I32),
        (3, U32),
        (4, Bool),
        (4, I32),
        (4, U32),
);

define_type!(Int => (I32, U32));

define_type!(Vec2 => (Vec2Bool, Vec2I32, Vec2U32));
define_type!(Vec3 => (Vec3Bool, Vec3I32, Vec3U32));
define_type!(Vec4 => (Vec4Bool, Vec4I32, Vec4U32));

define_type!(VecBool => (Vec2Bool, Vec3Bool, Vec4Bool));
define_type!(VecI32 => (Vec2I32, Vec3I32, Vec4I32));
define_type!(VecU32 => (Vec2U32, Vec3U32, Vec4U32));
define_type!(VecInt => (VecI32, VecU32));

define_type!(Scalar => (Bool, Int));
define_type!(Vec => (Vec2, Vec3, Vec4));

define_type!(Unconstrained => (Scalar, Vec));

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
        }
    }
}

impl TypeConstraints {
    pub fn empty() -> Self {
        TypeConstraints(HashTrieSetSync::new_sync())
    }

    pub fn any_of(types: impl IntoIterator<Item = DataType>) -> Self {
        let mut set = HashTrieSetSync::new_sync();

        for t in types {
            set.insert_mut(t);
        }

        TypeConstraints(set)
    }

    pub fn union(&self, other: &TypeConstraints) -> TypeConstraints {
        let mut new = self.clone();
        new.insert_all(other);
        new
    }

    pub fn contains(&self, t: DataType) -> bool {
        self.0.contains(&t)
    }

    pub fn intersects(&self, other: &TypeConstraints) -> bool {
        !self.intersection(other).0.is_empty()
    }

    pub fn intersection(&self, other: &TypeConstraints) -> TypeConstraints {
        let mut intersection = self.0.clone();

        for t in self.0.iter() {
            if !other.0.contains(t) {
                intersection.remove_mut(t);
            }
        }

        TypeConstraints(intersection)
    }

    pub fn select(&self, rng: &mut impl Rng) -> DataType {
        log::info!("selecting type from {:?}", self);
        *self.0.iter().choose(rng).unwrap()
    }

    pub fn insert(&mut self, t: DataType) {
        self.0.insert_mut(t);
    }

    pub fn insert_all(&mut self, other: &TypeConstraints) {
        for t in other.0.iter().copied() {
            self.insert(t);
        }
    }
}

impl From<DataType> for TypeConstraints {
    fn from(t: DataType) -> Self {
        TypeConstraints::any_of(iter::once(t))
    }
}

impl From<&DataType> for TypeConstraints {
    fn from(t: &DataType) -> Self {
        TypeConstraints::any_of(iter::once(*t))
    }
}
