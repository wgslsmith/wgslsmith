use crate::{DataType, ScalarType};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, strum::AsRefStr, strum::EnumIter, strum::EnumString,
)]
#[strum(serialize_all = "camelCase")]
pub enum BuiltinFn {
    Abs,
    Acos,
    Acosh,
    All,
    Any,
    ArrayLength,
    Asin,
    Asinh,
    Atan,
    Atanh,
    Atan2,
    AtomicAdd,
    AtomicLoad,
    Ceil,
    Clamp,
    Cos,
    Cosh,
    CountLeadingZeros,
    CountOneBits,
    CountTrailingZeros,
    Cross,
    Degrees,
    Distance,
    Dot,
    Exp,
    Exp2,
    ExtractBits,
    FaceForward,
    FirstLeadingBit,
    FirstTrailingBit,
    Floor,
    Fma,
    Fract,
    InsertBits,
    InverseSqrt,
    Ldexp,
    Length,
    Log,
    Log2,
    Max,
    Min,
    Mix,
    Normalize,
    Pow,
    QuantizeToF16,
    Radians,
    Reflect,
    Refract,
    ReverseBits,
    Round,
    Select,
    ShiftLeft,
    ShiftRight,
    Sign,
    Sin,
    Sinh,
    Smoothstep,
    Sqrt,
    Step,
    Tan,
    Tanh,
    Trunc,
}

impl BuiltinFn {
    /// Determines the return type for a builtin function, given argument types.
    ///
    /// Note that this only does the bare minimum work for overload resolution and does not do any
    /// actual validation/type checking. In most cases it only looks at the first argument and
    /// doesn't even validate if the correct number of arguments have been supplied.
    pub fn return_type<'a>(
        &self,
        mut params: impl Iterator<Item = &'a DataType>,
    ) -> Option<DataType> {
        use BuiltinFn::*;
        use ScalarType::*;

        let mut param = |n| {
          let mut p = params.next();
          for _ in 1..n {
            p = params.next();
          }
          p.map(DataType::dereference).cloned()
        };

        let ret = match self {
            Abs => param(1)?,
            Acos => param(1)?,
            Acosh => param(1)?,
            Asin => param(1)?,
            Asinh => param(1)?,
            Atan => param(1)?,
            Atanh => param(1)?,
            Atan2 => param(1)?,
            All => Bool.into(),
            Any => Bool.into(),
            ArrayLength => U32.into(),
            AtomicAdd => param(2)?,
            AtomicLoad => param(1)?,
            Ceil => param(1)?,
            Clamp => param(1)?,
            Cos => param(1)?,
            Cosh => param(1)?,
            CountLeadingZeros => param(1)?,
            CountOneBits => param(1)?,
            CountTrailingZeros => param(1)?,
            Cross => param(1)?,
            Degrees => param(1)?,
            Distance => F32.into(),
            Dot => param(1)?.as_scalar()?.into(),
            ExtractBits => param(1)?,
            Exp => param(1)?,
            Exp2 => param(1)?,
            FaceForward => param(1)?,
            FirstLeadingBit => param(1)?,
            FirstTrailingBit => param(1)?,
            Floor => param(1)?,
            Fma => param(1)?,
            Fract => param(1)?,
            InsertBits => param(1)?,
            InverseSqrt => param(1)?,
            Ldexp => param(1)?,
            Length => F32.into(),
            Log => param(1)?,
            Log2 => param(1)?,
            Max => param(1)?,
            Min => param(1)?,
            Mix => param(1)?,
            Normalize => param(1)?,
            Pow => param(1)?,
            QuantizeToF16 => param(1)?,
            Radians => param(1)?,
            Reflect => param(1)?,
            Refract => param(1)?,
            ReverseBits => param(1)?,
            Round => param(1)?,
            Select => param(1)?,
            ShiftLeft => param(1)?,
            ShiftRight => param(1)?,
            Sign => param(1)?,
            Sin => param(1)?,
            Sinh => param(1)?,
            Smoothstep => param(1)?,
            Sqrt => param(1)?,
            Step => param(1)?,
            Tan => param(1)?,
            Tanh => param(1)?,
            Trunc => param(1)?,
        };

        Some(ret)
    }
}
