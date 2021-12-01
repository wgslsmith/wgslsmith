use std::fmt::Display;

use rand::Rng;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TypeConstraints(u32);

impl TypeConstraints {
    pub const BOOL: TypeConstraints = TypeConstraints(1);
    pub const SINT: TypeConstraints = TypeConstraints(2);
    pub const UINT: TypeConstraints = TypeConstraints(4);

    pub const INT: TypeConstraints = TypeConstraints::SINT.union(TypeConstraints::UINT);
    pub const UNCONSTRAINED: TypeConstraints = TypeConstraints::BOOL.union(TypeConstraints::INT);

    pub fn any_of(i: impl IntoIterator<Item = DataType>) -> Self {
        let mut v = 0;

        for t in i {
            v |= t as u32;
        }

        TypeConstraints(v)
    }

    pub const fn union(self, other: TypeConstraints) -> TypeConstraints {
        TypeConstraints(self.0 | other.0)
    }

    pub const fn contains(self, other: TypeConstraints) -> bool {
        self.intersection(other).is_some()
    }

    pub const fn intersection(self, other: TypeConstraints) -> Option<TypeConstraints> {
        let intersection = self.0 & other.0;
        if intersection == 0 {
            None
        } else {
            Some(TypeConstraints(intersection))
        }
    }

    pub fn select(self, rng: &mut impl Rng) -> DataType {
        debug_assert_ne!(self.0, 0);

        let n = rng.gen_range(0..self.0.count_ones());
        let mut j = 0;

        for i in 0..32 {
            if self.0 & (1 << i) != 0 {
                if j == n {
                    return (1 << i).try_into().unwrap();
                } else {
                    j += 1;
                }
            }
        }

        // This should be unreachable as long as the constraints are never empty
        // i.e. self.0 != 0
        unreachable!()
    }
}

impl From<DataType> for TypeConstraints {
    fn from(t: DataType) -> Self {
        TypeConstraints(t as u32)
    }
}
