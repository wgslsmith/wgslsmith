use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{FnDecl, StructDecl};
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::Options;

use super::utils;

pub struct Context {
    pub types: RefCell<TypeContext>,
    pub fns: RefCell<FnContext>,
}

impl Context {
    pub fn new(options: Rc<Options>) -> Context {
        Context {
            types: RefCell::new(TypeContext::new()),
            fns: RefCell::new(FnContext::new(options)),
        }
    }
}

pub type FnSig = (String, Vec<DataType>, Option<DataType>);

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
            SelectionFilter::Any => &[ScalarType::I32, ScalarType::U32, ScalarType::Bool],
            SelectionFilter::HostShareable => &[ScalarType::I32, ScalarType::U32],
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

pub struct FnContext {
    map: HashMap<DataType, Vec<Rc<FnSig>>>,
    impls: Vec<FnDecl>,
    count: u32,
}

impl FnContext {
    pub fn new(options: Rc<Options>) -> Self {
        let sigs = ast::gen_builtin_fns(options.enabled_fns.iter().map(String::as_str))
            .into_iter()
            .map(Rc::new)
            .collect::<Vec<_>>();

        let mut map = HashMap::<_, Vec<_>>::new();
        for sig in &sigs {
            if let Some(ty) = sig.2.clone() {
                for key in iter::once(ty.clone()).chain(utils::accessible_types_of(&ty)) {
                    map.entry(key).or_default().push(sig.clone());
                }
            }
        }

        FnContext {
            map,
            impls: vec![],
            count: 0,
        }
    }

    pub fn len(&self) -> u32 {
        self.count
    }

    pub fn contains_type(&self, ty: &DataType) -> bool {
        self.map.contains_key(ty)
    }

    pub fn select(&self, rng: &mut impl Rng, return_ty: &DataType) -> Option<Rc<FnSig>> {
        self.map
            .get(return_ty)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .choose(rng)
            .cloned()
    }

    pub fn insert(&mut self, def: FnDecl) -> Rc<FnSig> {
        let sig = Rc::new((
            def.name.clone(),
            def.inputs
                .iter()
                .map(|param| param.data_type.clone())
                .collect(),
            def.output.as_ref().map(|ret| ret.data_type.clone()),
        ));

        if let Some(ty) = sig.2.clone() {
            for key in iter::once(ty.clone()).chain(utils::accessible_types_of(&ty)) {
                self.map.entry(key).or_default().push(sig.clone());
            }
        }

        self.impls.push(def);

        sig
    }

    pub fn next_fn(&mut self) -> String {
        self.count += 1;
        format!("func_{}", self.count)
    }

    pub fn into_fns(self) -> Vec<FnDecl> {
        self.impls
    }
}
