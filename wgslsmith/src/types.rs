use std::fmt::Display;
use std::iter;

use once_cell::sync::OnceCell;
use rand::prelude::IteratorRandom;
use rand::Rng;
use rpds::HashTrieSetSync;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DataType {
    Bool = 1,
    SInt = 2,
    UInt = 4,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DataType::Bool => "bool",
            DataType::SInt => "i32",
            DataType::UInt => "u32",
        })
    }
}

impl TryFrom<u32> for DataType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DataType::Bool),
            2 => Ok(DataType::SInt),
            4 => Ok(DataType::UInt),
            _ => Err(()),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TypeConstraints(HashTrieSetSync<DataType>);

macro_rules! define_type {
    ($type:ident) => {
        define_type!($type => ($type));
    };

    ($name:ident => ($($type:ident),*)) => {
        paste::paste!{
            static [<$name:snake:upper>]: OnceCell<TypeConstraints> = OnceCell::new();
            impl TypeConstraints {
                #[allow(non_snake_case)]
                pub fn [<$name>]() -> &'static TypeConstraints {
                    [<$name:snake:upper>].get_or_init(||{
                        TypeConstraints({
                            let mut set = HashTrieSetSync::new_sync();
                            $(set.insert_mut(DataType::$type);)*
                            set
                        })
                    })
                }
            }
        }
    };
}

define_type!(Bool);
define_type!(SInt);
define_type!(UInt);

define_type!(Unconstrained => (Bool, SInt, UInt));
define_type!(Int => (SInt, UInt));

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
        let mut new = self.0.clone();

        for t in other.0.iter().copied() {
            new.insert_mut(t);
        }

        TypeConstraints(new)
    }

    pub fn intersects(&self, other: &TypeConstraints) -> bool {
        self.intersection(other).is_some()
    }

    pub fn intersection(&self, other: &TypeConstraints) -> Option<TypeConstraints> {
        let mut intersection = self.0.clone();

        for t in self.0.iter() {
            if !other.0.contains(t) {
                intersection.remove_mut(t);
            }
        }

        if intersection.is_empty() {
            None
        } else {
            Some(TypeConstraints(intersection))
        }
    }

    pub fn select(&self, rng: &mut impl Rng) -> DataType {
        *self.0.iter().choose(rng).unwrap()
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
