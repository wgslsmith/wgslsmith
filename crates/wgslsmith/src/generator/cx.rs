use std::collections::{HashMap, HashSet};
use std::iter;
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{BuiltinFn, FnDecl, StructDecl};
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::Options;

use super::utils;

pub struct Context {
    pub types: TypeContext,
    pub fns: FnContext,
}

impl Context {
    pub fn new(options: Rc<Options>) -> Context {
        Context {
            types: TypeContext::new(),
            fns: FnContext::new(options),
        }
    }
}

#[derive(Debug)]
pub struct FnSignature {
    pub ident: String,
    pub params: Vec<DataType>,
    pub return_type: Option<DataType>,
}

pub struct TypeContext {
    types: Vec<Rc<StructDecl>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionFilter {
    Any,
    HostShareable,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext { types: Vec::new() }
    }

    pub fn insert(&mut self, decl: Rc<StructDecl>) {
        self.types.push(decl);
    }

    pub fn select(&self, rng: &mut impl Rng) -> DataType {
        self.select_with_filter(rng, SelectionFilter::Any)
    }

    pub fn select_with_filter(&self, rng: &mut impl Rng, filter: SelectionFilter) -> DataType {
        let allowed_scalars: &[ScalarType] = match filter {
            SelectionFilter::Any => &[
                ScalarType::I32,
                ScalarType::U32,
                ScalarType::F32,
                ScalarType::Bool,
            ],
            SelectionFilter::HostShareable => &[ScalarType::I32, ScalarType::U32, ScalarType::F32],
        };

        enum DataTypeKind {
            Scalar,
            Vector,
            User,
        }

        let allowed: &[DataTypeKind] =
            if filter == SelectionFilter::HostShareable || self.types.is_empty() {
                &[DataTypeKind::Scalar, DataTypeKind::Vector]
            } else {
                &[
                    DataTypeKind::Scalar,
                    DataTypeKind::Vector,
                    DataTypeKind::User,
                ]
            };

        match allowed.choose(rng).unwrap() {
            DataTypeKind::Scalar => DataType::Scalar(allowed_scalars.choose(rng).copied().unwrap()),
            DataTypeKind::Vector => DataType::Vector(
                rng.gen_range(2..=4),
                allowed_scalars.choose(rng).copied().unwrap(),
            ),
            DataTypeKind::User => DataType::Struct(self.types.choose(rng).cloned().unwrap()),
        }
    }

    pub fn into_structs(self) -> Vec<Rc<StructDecl>> {
        self.types
    }
}

#[derive(Debug)]
pub struct Overload {
    pub params: Vec<DataType>,
    pub return_type: DataType,
}

#[derive(Debug)]
pub enum Func {
    Builtin(BuiltinFn, Overload),
    User(FnSignature),
}

pub struct FnContext {
    map: HashMap<DataType, Vec<Rc<Func>>>,
    decls: Vec<FnDecl>,
    count: u32,
}

impl FnContext {
    pub fn new(options: Rc<Options>) -> Self {
        FnContext {
            map: gen_builtins(&options.enabled_fns),
            decls: vec![],
            count: 0,
        }
    }

    pub fn len(&self) -> u32 {
        self.count
    }

    pub fn contains_type(&self, ty: &DataType) -> bool {
        self.map.contains_key(ty)
    }

    pub fn select(&self, rng: &mut impl Rng, return_ty: &DataType) -> Option<Rc<Func>> {
        self.map
            .get(return_ty)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .choose(rng)
            .cloned()
    }

    pub fn insert(&mut self, decl: FnDecl) -> Rc<Func> {
        let sig = FnSignature {
            ident: decl.name.clone(),
            params: decl
                .inputs
                .iter()
                .map(|param| param.data_type.clone())
                .collect(),
            return_type: decl.output.as_ref().map(|ret| ret.data_type.clone()),
        };

        let return_type = sig.return_type.clone();
        let func = Rc::new(Func::User(sig));

        if let Some(ty) = return_type {
            for key in iter::once(ty.clone()).chain(utils::accessible_types_of(&ty)) {
                self.map.entry(key).or_default().push(func.clone());
            }
        }

        self.decls.push(decl);

        func
    }

    pub fn next_fn(&mut self) -> String {
        self.count += 1;
        format!("func_{}", self.count)
    }

    pub fn into_fns(self) -> Vec<FnDecl> {
        self.decls
    }
}

fn vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    (2..=4).map(move |n| DataType::Vector(n, ty))
}

fn scalar_and_vectors_of(ty: ScalarType) -> impl Iterator<Item = DataType> {
    std::iter::once(DataType::Scalar(ty)).chain(vectors_of(ty))
}

fn gen_builtins(enabled: &[BuiltinFn]) -> HashMap<DataType, Vec<Rc<Func>>> {
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
