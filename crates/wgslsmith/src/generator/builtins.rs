use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use ast::{BuiltinFn, DataType, ScalarType};

use crate::generator::cx::Func;

use super::cx::Overload;

fn vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    (2..=4).map(move |n| DataType::Vector(n, ty))
}

fn scalar_and_vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    std::iter::once(DataType::Scalar(ty)).chain(vectors_of(ty))
}

pub fn gen_builtins(enabled: &[BuiltinFn]) -> HashMap<DataType, Vec<Rc<Func>>> {
    use BuiltinFn::*;
    use DataType::*;
    use ScalarType::*;

    let enabled: HashSet<BuiltinFn> = HashSet::from_iter(enabled.iter().copied());
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

    for s_ty in [ScalarType::I32, ScalarType::U32] {
        for ty in scalar_and_vectors_of(s_ty) {
            map.add(Clamp, [ty.clone(), ty.clone(), ty.clone()], ty.clone());

            for builtin in [
                Abs,
                CountOneBits,
                ReverseBits,
                FirstLeadingBit,
                FirstTrailingBit,
            ] {
                map.add(builtin, [ty.clone()], ty.clone());
            }

            for builtin in [Max, Min] {
                map.add(builtin, [ty.clone(), ty.clone()], ty.clone());
            }

            // TODO: Enable functions below once they've been implemented in naga and tint
            // https://github.com/gfx-rs/naga/issues/1824
            // https://github.com/gfx-rs/naga/issues/1929

            for builtin in [CountLeadingZeros, CountTrailingZeros] {
                if enabled.contains(&builtin) {
                    map.add(builtin, [ty.clone()], ty.clone());
                }
            }

            if enabled.contains(&ExtractBits) {
                map.add(
                    ExtractBits,
                    [ty.clone(), U32.into(), U32.into()],
                    ty.clone(),
                );
            }

            if enabled.contains(&InsertBits) {
                map.add(
                    InsertBits,
                    [ty.clone(), ty.clone(), U32.into(), U32.into()],
                    ty.clone(),
                );
            }
        }

        for ty in vectors_of(s_ty) {
            map.add(Dot, [ty.clone(), ty.clone()], s_ty);
        }
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
