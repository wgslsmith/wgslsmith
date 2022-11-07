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

        let mut first_param = || params.next().map(DataType::dereference).cloned();

        let ret = match self {
            Abs => first_param()?,
            Acos => first_param()?,
            Acosh => first_param()?,
            Asin => first_param()?,
            Asinh => first_param()?,
            Atan => first_param()?,
            Atanh => first_param()?,
            Atan2 => first_param()?,
            All => Bool.into(),
            Any => Bool.into(),
            ArrayLength => U32.into(),
            AtomicAdd => U32.into(), // TODO: Fix so we can use atomic ops on other atomic types
            Ceil => first_param()?,
            Clamp => first_param()?,
            Cos => first_param()?,
            Cosh => first_param()?,
            CountLeadingZeros => first_param()?,
            CountOneBits => first_param()?,
            CountTrailingZeros => first_param()?,
            Cross => first_param()?,
            Degrees => first_param()?,
            Distance => F32.into(),
            Dot => first_param()?.as_scalar()?.into(),
            ExtractBits => first_param()?,
            Exp => first_param()?,
            Exp2 => first_param()?,
            FaceForward => first_param()?,
            FirstLeadingBit => first_param()?,
            FirstTrailingBit => first_param()?,
            Floor => first_param()?,
            Fma => first_param()?,
            Fract => first_param()?,
            InsertBits => first_param()?,
            InverseSqrt => first_param()?,
            Ldexp => first_param()?,
            Length => F32.into(),
            Log => first_param()?,
            Log2 => first_param()?,
            Max => first_param()?,
            Min => first_param()?,
            Mix => first_param()?,
            Normalize => first_param()?,
            Pow => first_param()?,
            QuantizeToF16 => first_param()?,
            Radians => first_param()?,
            Reflect => first_param()?,
            Refract => first_param()?,
            ReverseBits => first_param()?,
            Round => first_param()?,
            Select => first_param()?,
            ShiftLeft => first_param()?,
            ShiftRight => first_param()?,
            Sign => first_param()?,
            Sin => first_param()?,
            Sinh => first_param()?,
            Smoothstep => first_param()?,
            Sqrt => first_param()?,
            Step => first_param()?,
            Tan => first_param()?,
            Tanh => first_param()?,
            Trunc => first_param()?,
        };

        Some(ret)
    }
}
