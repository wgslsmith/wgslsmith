pub mod types;
pub mod writer;

mod expr;
mod func;
mod globals;
mod stmt;
mod structs;

pub use expr::*;
pub use func::*;
pub use globals::*;
pub use stmt::*;
pub use structs::*;

use std::collections::HashSet;
use std::rc::Rc;

pub use types::{DataType, ScalarType};

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub structs: Vec<Rc<StructDecl>>,
    pub consts: Vec<GlobalConstDecl>,
    pub vars: Vec<GlobalVarDecl>,
    pub functions: Vec<FnDecl>,
}

fn vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    (2..=4).map(move |n| DataType::Vector(n, ty))
}

fn scalar_and_vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    std::iter::once(DataType::Scalar(ty)).chain(vectors_of(ty))
}

pub fn gen_builtin_fns<'a>(
    enabled: impl Iterator<Item = &'a str>,
) -> Vec<(String, Vec<DataType>, Option<DataType>)> {
    let mut fns = Vec::new();
    let enabled = enabled.collect::<HashSet<_>>();

    for ty in vectors_of(ScalarType::Bool) {
        fns.push((
            "all".to_owned(),
            vec![ty.clone()],
            Some(DataType::Scalar(ScalarType::Bool)),
        ));

        fns.push((
            "any".to_owned(),
            vec![ty.clone()],
            Some(DataType::Scalar(ScalarType::Bool)),
        ));
    }

    for s_ty in [ScalarType::Bool, ScalarType::I32, ScalarType::U32] {
        for ty in scalar_and_vectors_of(s_ty) {
            fns.push((
                "select".to_owned(),
                vec![ty.clone(), ty.clone(), DataType::Scalar(ScalarType::Bool)],
                Some(ty),
            ));
        }

        for n in 2..=4 {
            fns.push((
                "select".to_owned(),
                vec![
                    DataType::Vector(n, s_ty),
                    DataType::Vector(n, s_ty),
                    DataType::Vector(n, ScalarType::Bool),
                ],
                Some(DataType::Vector(n, s_ty)),
            ));
        }
    }

    for s_ty in [ScalarType::I32, ScalarType::U32] {
        for ty in scalar_and_vectors_of(s_ty) {
            fns.push((
                "clamp".to_owned(),
                vec![ty.clone(), ty.clone(), ty.clone()],
                Some(ty.clone()),
            ));

            // TODO: Enable functions below once they've been implemented in naga and tint

            for ident in ["abs", "countOneBits", "reverseBits"] {
                fns.push((ident.to_owned(), vec![ty.clone()], Some(ty.clone())));
            }

            for ident in [
                "countLeadingZeros",
                "countTrailingZeros",
                "firstBitHigh",
                "firstBitLow",
            ] {
                if enabled.contains(ident) {
                    fns.push((ident.to_owned(), vec![ty.clone()], Some(ty.clone())));
                }
            }

            if enabled.contains("extractBits") {
                fns.push((
                    "extractBits".to_owned(),
                    vec![
                        ty.clone(),
                        DataType::Scalar(ScalarType::U32),
                        DataType::Scalar(ScalarType::U32),
                    ],
                    Some(ty.clone()),
                ));
            }

            if enabled.contains("insertBits") {
                fns.push((
                    "insertBits".to_owned(),
                    vec![
                        ty.clone(),
                        ty.clone(),
                        DataType::Scalar(ScalarType::U32),
                        DataType::Scalar(ScalarType::U32),
                    ],
                    Some(ty.clone()),
                ));
            }

            for ident in ["max", "min"] {
                fns.push((
                    ident.to_owned(),
                    vec![ty.clone(), ty.clone()],
                    Some(ty.clone()),
                ));
            }
        }

        for ty in vectors_of(s_ty) {
            fns.push((
                "dot".to_owned(),
                vec![ty.clone(), ty.clone()],
                Some(DataType::Scalar(s_ty)),
            ));
        }
    }

    fns
}
