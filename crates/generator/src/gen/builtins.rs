use std::collections::HashMap;
use std::rc::Rc;

use ast::{BuiltinFn, DataType, ScalarType};

use super::cx::Overload;
use crate::gen::cx::Func;
use crate::Options;

fn vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    (2..=4).map(move |n| DataType::Vector(n, ty))
}

fn scalar_and_vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    std::iter::once(DataType::Scalar(ty)).chain(vectors_of(ty))
}

pub const TINT_EXTRAS: &[BuiltinFn] = {
    use BuiltinFn::*;
    &[CountLeadingZeros, CountTrailingZeros, Refract]
};

pub fn gen_builtins(options: &Options) -> HashMap<DataType, Vec<Rc<Func>>> {
    use BuiltinFn::*;
    use DataType::*;
    use ScalarType::*;

    let mut map = HashMap::<DataType, Vec<Rc<Func>>>::new();

    let mut numeric_scalars = vec![I32, U32, F32];
    let mut all_scalars = vec![Bool, I32, U32, F32];
    let mut float_scalars = vec![F32];

    if options.enable_f16() {
        numeric_scalars.push(F16);
        all_scalars.push(F16);
        float_scalars.push(F16);
    }

    for s_ty in numeric_scalars.clone() {
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Abs, [ty.clone()], ty.clone());
            map.add(Clamp, [ty.clone(), ty.clone(), ty.clone()], ty.clone());
            map.add(Max, [ty.clone(), ty.clone()], ty.clone());
            map.add(Min, [ty.clone(), ty.clone()], ty.clone());
        }

        for ty in vectors_of(s_ty) {
            map.add(Dot, [ty.clone(), ty.clone()], s_ty);
        }
    }

    for ty in scalar_and_vectors_of(I32) {
        map.add(Sign, [ty.clone()], ty.clone());
    }

    for ty in vectors_of(Bool) {
        map.add(All, [ty.clone()], Bool);
        map.add(Any, [ty.clone()], Bool);
    }

    for s_ty in all_scalars.clone() {
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Select, [ty.clone(), ty.clone(), Bool.into()], ty.clone());
        }

        for n in 2..=4 {
            map.add(
                Select,
                [Vector(n, s_ty), Vector(n, s_ty), Vector(n, Bool)],
                Vector(n, s_ty),
            );
        }
    }

    for s_ty in [I32, U32] {
        let atomic_ty = DataType::Atomic(s_ty);
        let ptr_ty = DataType::Ptr(ast::types::MemoryViewType::new(
            atomic_ty,
            ast::StorageClass::WorkGroup,
        ));

        for builtin in [
            AtomicAdd,
            AtomicAnd,
            AtomicExchange,
            AtomicMax,
            AtomicMin,
            AtomicOr,
            AtomicSub,
            AtomicXor,
        ] {
            map.add(builtin, [ptr_ty.clone(), s_ty.into()], s_ty);
        }
        map.add(AtomicLoad, [ptr_ty.clone()], s_ty);
        map.add(
            AtomicCompareExchangeWeak,
            [ptr_ty.clone(), s_ty.into(), s_ty.into()],
            DataType::AtomicCompareExchangeResult(s_ty),
        );

        for ty in scalar_and_vectors_of(s_ty) {
            for builtin in [
                CountOneBits,
                CountLeadingZeros,
                CountTrailingZeros,
                ReverseBits,
                FirstLeadingBit,
                FirstTrailingBit,
            ] {
                map.add(builtin, [ty.clone()], ty.clone());
            }

            map.add(
                ExtractBits,
                [ty.clone(), U32.into(), U32.into()],
                ty.clone(),
            );

            map.add(
                InsertBits,
                [ty.clone(), ty.clone(), U32.into(), U32.into()],
                ty.clone(),
            );
        }
    }

    for s_ty in float_scalars.clone() {
        for ty in scalar_and_vectors_of(s_ty) {
            for builtin in [Ceil, Exp, Exp2, Floor, Fract, Round, Saturate, Sign, Trunc] {
                map.add(builtin, [ty.clone()], ty.clone());
            }

            if options.unstable_float {
                for builtin in [
                    Acos,
                    Acosh,
                    Asin,
                    Asinh,
                    Atan,
                    Atanh,
                    Cos,
                    Cosh,
                    Degrees,
                    InverseSqrt,
                    Log,
                    Log2,
                    Radians,
                    Sin,
                    Sinh,
                    Sqrt,
                    Tan,
                    Tanh,
                ] {
                    map.add(builtin, [ty.clone()], ty.clone());
                }

                if s_ty == F32 {
                    map.add(QuantizeToF16, [ty.clone()], ty.clone());
                }
            }

            map.add(Step, [ty.clone(), ty.clone()], ty.clone());

            if options.unstable_float {
                for builtin in [Pow, Atan2] {
                    map.add(builtin, [ty.clone(), ty.clone()], ty.clone());
                }

                for builtin in [Fma, Mix, Smoothstep] {
                    map.add(builtin, [ty.clone(), ty.clone(), ty.clone()], ty.clone());
                }

                map.add(Distance, [ty.clone(), ty.clone()], s_ty);
                map.add(Ldexp, [ty.clone(), ty.map(I32)], ty.clone());
                map.add(Length, [ty.clone()], s_ty);
            }
        }

        if options.unstable_float {
            map.add(Cross, [Vector(3, s_ty), Vector(3, s_ty)], Vector(3, s_ty));

            for ty in vectors_of(s_ty) {
                map.add(Normalize, [ty.clone()], ty.clone());

                map.add(
                    FaceForward,
                    [ty.clone(), ty.clone(), ty.clone()],
                    ty.clone(),
                );

                map.add(Reflect, [ty.clone(), ty.clone()], ty.clone());

                map.add(Refract, [ty.clone(), ty.clone(), s_ty.into()], ty.clone());
            }
        }
    }

    if options.collectives() {
        for s_ty in numeric_scalars.clone() {
            for ty in scalar_and_vectors_of(s_ty) {
                map.add(SubgroupAdd, [ty.clone()], ty.clone());
                map.add(SubgroupExclusiveAdd, [ty.clone()], ty.clone());
                map.add(SubgroupInclusiveAdd, [ty.clone()], ty.clone());
                map.add(SubgroupMul, [ty.clone()], ty.clone());
                map.add(SubgroupExclusiveMul, [ty.clone()], ty.clone());
                map.add(SubgroupInclusiveMul, [ty.clone()], ty.clone());
                map.add(SubgroupMin, [ty.clone()], ty.clone());
                map.add(SubgroupMax, [ty.clone()], ty.clone());

                //map.add(SubgroupBroadcast, [ty.clone(), U32.into()], ty.clone());
                map.add(SubgroupBroadcastFirst, [ty.clone()], ty.clone());
                map.add(SubgroupShuffle, [ty.clone(), U32.into()], ty.clone());
                // map.add(SubgroupShuffleDown, [ty.clone(), U32.into()], ty.clone());
                // map.add(SubgroupShuffleUp, [ty.clone(), U32.into()], ty.clone());
                // map.add(SubgroupShuffleXor, [ty.clone(), U32.into()], ty.clone());
            }

            let wg_ptr_ty = DataType::Ptr(ast::types::MemoryViewType::new(
                DataType::Scalar(s_ty),
                ast::StorageClass::WorkGroup,
            ));
            map.add(WorkgroupUniformLoad, [wg_ptr_ty], DataType::Scalar(s_ty));
        }

        for s_ty in [I32, U32] {
            for ty in scalar_and_vectors_of(s_ty) {
                map.add(SubgroupAnd, [ty.clone()], ty.clone());
                map.add(SubgroupOr, [ty.clone()], ty.clone());
                map.add(SubgroupXor, [ty.clone()], ty.clone());
            }

            let atomic_ty = DataType::Atomic(s_ty);
            let wg_ptr_atomic_ty = DataType::Ptr(ast::types::MemoryViewType::new(
                atomic_ty,
                ast::StorageClass::WorkGroup,
            ));
            map.add(WorkgroupUniformLoad, [wg_ptr_atomic_ty], s_ty);
        }

        map.add(SubgroupAll, [Bool.into()], Bool);
        map.add(SubgroupAny, [Bool.into()], Bool);
        map.add(SubgroupBallot, [Bool.into()], DataType::Vector(4, U32));
        //map.add(SubgroupElect, vec![], Bool); // not supported in dawn

        let wg_ptr_bool_ty = DataType::Ptr(ast::types::MemoryViewType::new(
            DataType::Scalar(Bool),
            ast::StorageClass::WorkGroup,
        ));
        map.add(
            WorkgroupUniformLoad,
            [wg_ptr_bool_ty],
            DataType::Scalar(Bool),
        );
    }

    map
}

trait HashMapExt {
    fn add(
        &mut self,
        builtin: BuiltinFn,
        params: impl Into<Vec<DataType>>,
        return_type: impl Into<DataType>,
    );
}

impl HashMapExt for HashMap<DataType, Vec<Rc<Func>>> {
    fn add(
        &mut self,
        builtin: BuiltinFn,
        params: impl Into<Vec<DataType>>,
        return_type: impl Into<DataType>,
    ) {
        let return_type = return_type.into();
        let func = Rc::new(Func::Builtin(
            builtin,
            Overload {
                params: params.into(),
                return_type: return_type.clone(),
            },
        ));

        for key in std::iter::once(return_type.clone())
            .chain(super::utils::accessible_types_of(&return_type))
        {
            self.entry(key).or_default().push(func.clone());
        }
    }
}
