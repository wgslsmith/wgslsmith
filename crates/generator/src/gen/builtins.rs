use std::collections::HashMap;
use std::rc::Rc;

use ast::{BuiltinFn, DataType, ScalarType};

use crate::gen::cx::Func;

use super::cx::Overload;

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

pub fn gen_builtins(_enabled: &[BuiltinFn]) -> HashMap<DataType, Vec<Rc<Func>>> {
    use BuiltinFn::*;
    use DataType::*;
    use ScalarType::*;

    let mut map = HashMap::<DataType, Vec<Rc<Func>>>::new();

    for s_ty in [I32, U32, F32] {
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Abs, [ty.clone()], ty);
        }
    }

    for ty in vectors_of(Bool) {
        map.add(All, [ty.clone()], Bool);
        map.add(Any, [ty.clone()], Bool);
    }

    for s_ty in [Bool, I32, U32, F32] {
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Select, [ty.clone(), ty.clone(), Bool.into()], ty);
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
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Clamp, [ty.clone(), ty.clone(), ty.clone()], ty.clone());

            for builtin in [
                Abs,
                CountOneBits,
                CountLeadingZeros,
                CountTrailingZeros,
                ReverseBits,
                FirstLeadingBit,
                FirstTrailingBit,
            ] {
                map.add(builtin, [ty.clone()], ty.clone());
            }

            for builtin in [Max, Min] {
                map.add(builtin, [ty.clone(), ty.clone()], ty.clone());
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

        for ty in vectors_of(s_ty) {
            map.add(Dot, [ty.clone(), ty.clone()], s_ty);
        }
    }

    for ty in scalar_and_vectors_of(F32) {
        for builtin in [
            // Acos - // TODO: recondition,
            // Acosh - not implemented in tint/naga,
            // Asin - // TODO: recondition,
            // Asinh - not implemnted in tint/naga,
            // Atan - // TODO: recondition,
            // Atanh - not implemented in tint/naga,
            Ceil, // Cos,
            // Cosh,
            // Degrees,
            Exp, Exp2, Floor, Fract,
            // InverseSqrt - // TODO: recondition,
            // Log - // TODO: recondition,
            // Log2 - // TODO: recondition,
            // QuantizeToF16 - // TODO: recondition,
            // Radians,
            Round, Saturate, Sign,
            // Sin,
            // Sinh,
            // Sqrt - // TODO: recondition,
            // Tan - // TODO: recondition,
            // Tanh - // TODO: recondition,
            Trunc,
        ] {
            map.add(builtin, [ty.clone()], ty.clone());
        }

        for builtin in [Max, Min /*, Pow */, Step] {
            map.add(builtin, [ty.clone(), ty.clone()], ty.clone());
        }

        // for builtin in [Fma, Mix, Smoothstep] {
        //     map.add(builtin, [ty.clone(), ty.clone(), ty.clone()], ty.clone());
        // }

        // map.add(Distance, [ty.clone(), ty.clone()], F32);
        // map.add(Ldexp, [ty.clone(), ty.map(I32)], ty.clone()); // https://github.com/gfx-rs/naga/issues/1908
        // map.add(Length, [ty.clone()], F32);
    }

    // map.add(Cross, [Vector(3, F32), Vector(3, F32)], Vector(3, F32));

    // for ty in vectors_of(F32) {
    //     map.add(
    //         FaceForward,
    //         [ty.clone(), ty.clone(), ty.clone()],
    //         ty.clone(),
    //     );

    //     map.add(Reflect, [ty.clone(), ty.clone()], ty.clone());

    //     // Unimplemented in naga
    //     if enabled.contains(&Refract) {
    //         map.add(Refract, [ty.clone(), ty.clone(), F32.into()], ty.clone());
    //     }
    // }

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
        self.entry(return_type.clone())
            .or_default()
            .push(Rc::new(Func::Builtin(
                builtin,
                Overload {
                    params: params.into(),
                    return_type,
                },
            )));
    }
}
