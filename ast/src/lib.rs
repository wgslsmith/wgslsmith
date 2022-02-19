pub mod types;

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
use std::fmt::Display;
use std::str::FromStr;

use types::{DataType, ScalarType};

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub structs: Vec<StructDecl>,
    pub vars: Vec<GlobalVarDecl>,
    pub functions: Vec<FnDecl>,
    pub entrypoint: FnDecl,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for decl in &self.structs {
            writeln!(f, "{}", decl)?;
        }

        for decl in &self.vars {
            writeln!(f, "{}", decl)?;
        }

        for decl in &self.functions {
            writeln!(f, "{}", decl)?;
        }

        self.entrypoint.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttrList<T>(pub Vec<Attr<T>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttrStyle {
    Cpp,
    Java,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Attr<T> {
    pub style: AttrStyle,
    pub attr: T,
}

impl FromStr for AttrStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cpp" => Ok(AttrStyle::Cpp),
            "java" => Ok(AttrStyle::Java),
            _ => Err("invalid attribute style - must be one of {cpp, java}"),
        }
    }
}

impl<T> FromIterator<Attr<T>> for AttrList<T> {
    fn from_iter<I: IntoIterator<Item = Attr<T>>>(iter: I) -> Self {
        AttrList(Vec::from_iter(iter))
    }
}

impl<T: Display> Display for AttrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for attr in &self.0 {
            writeln!(f, "{}", attr)?
        }

        Ok(())
    }
}

impl<T: Display> Display for Attr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.style {
            AttrStyle::Cpp => write!(f, "[[{}]]", self.attr),
            AttrStyle::Java => write!(f, "@{}", self.attr),
        }
    }
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

            for ident in ["abs"] {
                fns.push((ident.to_owned(), vec![ty.clone()], Some(ty.clone())));
            }

            for ident in [
                "countLeadingZeros",
                "countOneBits",
                "countTrailingZeros",
                "firstBitHigh",
                "firstBitLow",
                "reverseBits",
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
