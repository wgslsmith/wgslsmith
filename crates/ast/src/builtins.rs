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
    AtomicAnd,
    AtomicCompareExchangeWeak,
    AtomicExchange,
    AtomicLoad,
    AtomicMax,
    AtomicMin,
    AtomicOr,
    AtomicStore,
    AtomicSub,
    AtomicXor,
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
    Saturate,
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

    // Subgroup
    SubgroupAdd,
    SubgroupAnd,
    SubgroupExclusiveAdd,
    SubgroupInclusiveAdd,
    SubgroupAll,
    SubgroupAny,
    SubgroupBallot,
    SubgroupBroadcast,
    SubgroupBroadcastFirst,
    SubgroupElect,
    SubgroupMax,
    SubgroupMin,
    SubgroupMul,
    SubgroupExclusiveMul,
    SubgroupInclusiveMul,
    SubgroupOr,
    SubgroupShuffle,
    SubgroupShuffleDown,
    SubgroupShuffleUp,
    SubgroupShuffleXor,
    SubgroupXor,

    // Synchronization
    StorageBarrier,
    TextureBarrier,
    WorkgroupBarrier,
    WorkgroupUniformLoad,
}

impl BuiltinFn {
    pub fn requires_uniformity(&self) -> bool {
        matches!(
            self,
            BuiltinFn::SubgroupAdd
                | BuiltinFn::SubgroupAnd
                | BuiltinFn::SubgroupExclusiveAdd
                | BuiltinFn::SubgroupInclusiveAdd
                | BuiltinFn::SubgroupAll
                | BuiltinFn::SubgroupAny
                | BuiltinFn::SubgroupBallot
                | BuiltinFn::SubgroupBroadcast
                | BuiltinFn::SubgroupBroadcastFirst
                | BuiltinFn::SubgroupElect
                | BuiltinFn::SubgroupMax
                | BuiltinFn::SubgroupMin
                | BuiltinFn::SubgroupMul
                | BuiltinFn::SubgroupExclusiveMul
                | BuiltinFn::SubgroupInclusiveMul
                | BuiltinFn::SubgroupOr
                | BuiltinFn::SubgroupShuffle
                | BuiltinFn::SubgroupShuffleDown
                | BuiltinFn::SubgroupShuffleUp
                | BuiltinFn::SubgroupShuffleXor
                | BuiltinFn::SubgroupXor
                | BuiltinFn::WorkgroupUniformLoad
                | BuiltinFn::StorageBarrier
                | BuiltinFn::TextureBarrier
                | BuiltinFn::WorkgroupBarrier
        )
    }
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
            AtomicAdd | AtomicAnd | AtomicExchange | AtomicLoad | AtomicMax | AtomicMin
            | AtomicOr | AtomicSub | AtomicXor => {
                let ty = first_param()?;
                if let DataType::Ptr(view) = ty {
                    if let DataType::Atomic(t) = view.inner.as_ref() {
                        DataType::Scalar(*t)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            AtomicCompareExchangeWeak => {
                let ty = first_param()?;
                if let DataType::Ptr(view) = ty {
                    if let DataType::Atomic(t) = view.inner.as_ref() {
                        DataType::AtomicCompareExchangeResult(*t)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            AtomicStore => return None,
            All => Bool.into(),
            Any => Bool.into(),
            ArrayLength => U32.into(),
            Ceil => first_param()?,
            Clamp => first_param()?,
            Cos => first_param()?,
            Cosh => first_param()?,
            CountLeadingZeros => first_param()?,
            CountOneBits => first_param()?,
            CountTrailingZeros => first_param()?,
            Cross => first_param()?,
            Degrees => first_param()?,
            Distance => first_param()?.as_scalar()?.into(),
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
            Length => first_param()?.as_scalar()?.into(),
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
            Saturate => first_param()?,
            Select => first_param()?,
            ShiftLeft => first_param()?,
            ShiftRight => first_param()?,
            Sign => first_param()?,
            Sin => first_param()?,
            Sinh => first_param()?,
            Smoothstep => first_param()?,
            Sqrt => first_param()?,
            Step => first_param()?,
            SubgroupBallot => DataType::Vector(4, U32),
            SubgroupBroadcast
            | SubgroupBroadcastFirst
            | SubgroupShuffle
            | SubgroupShuffleXor
            | SubgroupShuffleUp
            | SubgroupShuffleDown => first_param()?,
            SubgroupAdd | SubgroupExclusiveAdd | SubgroupInclusiveAdd | SubgroupMul
            | SubgroupExclusiveMul | SubgroupInclusiveMul | SubgroupMin | SubgroupMax
            | SubgroupAnd | SubgroupOr | SubgroupXor => first_param()?,
            SubgroupAll | SubgroupAny | SubgroupElect => Bool.into(),
            StorageBarrier | TextureBarrier | WorkgroupBarrier => return None,
            Tan => first_param()?,
            Tanh => first_param()?,
            Trunc => first_param()?,
            WorkgroupUniformLoad => {
                let ty = first_param()?;
                if let DataType::Ptr(view) = ty {
                    if let DataType::Atomic(t) = view.inner.as_ref() {
                        DataType::Scalar(*t)
                    } else {
                        view.inner.as_ref().clone()
                    }
                } else {
                    return None;
                }
            }
        };

        Some(ret)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum BuiltinValue {
    VertexIndex,
    InstanceIndex,
    Position,
    FrontFacing,
    FragDepth,
    LocalInvocationId,
    LocalInvocationIndex,
    GlobalInvocationId,
    WorkgroupId,
    NumWorkgroups,
    SampleIndex,
    SampleMask,
    ClipDistances,
    PrimitiveIndex,
    GlobalInvocationIndex,
    WorkgroupIndex,
    SubgroupInvocationId,
    SubgroupSize,
    SubgroupId,
    NumSubgroups,
}
