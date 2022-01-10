use std::fmt;

use ast::types::{DataType, ScalarType};
use once_cell::sync::OnceCell;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::HashTrieSetSync;

use crate::define_type;

#[derive(Clone, PartialEq, Eq)]
pub struct TypeConstraints(HashTrieSetSync<DataType, crate::BuildFxHasher>);

impl fmt::Debug for TypeConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
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

impl TypeConstraints {
    pub fn empty() -> Self {
        TypeConstraints(HashTrieSetSync::new_with_hasher_with_ptr_kind(
            crate::BuildFxHasher,
        ))
    }

    pub fn any_of(types: impl IntoIterator<Item = DataType>) -> Self {
        let mut set = HashTrieSetSync::new_with_hasher_with_ptr_kind(crate::BuildFxHasher);

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
        for t in other.0.iter() {
            if self.0.contains(t) {
                return true;
            }
        }

        false
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
        self.0.iter().choose(rng).unwrap().clone()
    }

    pub fn insert(&mut self, t: DataType) {
        self.0.insert_mut(t);
    }

    pub fn insert_all(&mut self, other: &TypeConstraints) {
        for t in other.0.iter().cloned() {
            self.insert(t);
        }
    }

    pub fn map_to_scalars(&self, types: &[ScalarType]) -> TypeConstraints {
        let mut result = TypeConstraints::empty();

        for t in self.0.iter() {
            match t {
                DataType::Scalar(_) => types
                    .iter()
                    .for_each(|t| result.insert(DataType::Scalar(*t))),

                DataType::Vector(n, _) => types
                    .iter()
                    .for_each(|t| result.insert(DataType::Vector(*n, *t))),

                _ => unimplemented!(),
            }
        }

        result
    }
}

pub trait DataTypeExt {
    fn to_constraints(&self) -> TypeConstraints;
}

impl DataTypeExt for DataType {
    fn to_constraints(&self) -> TypeConstraints {
        TypeConstraints::any_of(std::iter::once(self.clone()))
    }
}
