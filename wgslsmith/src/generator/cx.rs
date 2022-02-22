use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{FnDecl, StructDecl, StructMember};
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::Rng;

use crate::Options;

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
    types: HashMap<String, StructDecl>,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            types: HashMap::new(),
        }
    }

    pub fn insert(&mut self, decl: StructDecl) {
        self.types.insert(decl.name.clone(), decl);
    }

    pub fn select(&self, rng: &mut impl Rng) -> DataType {
        let scalar_ty = [ScalarType::I32, ScalarType::U32, ScalarType::Bool]
            .choose(rng)
            .copied()
            .unwrap();

        enum DataTypeKind {
            Scalar,
            Vector,
            User,
        }

        let allowed: &[DataTypeKind] = if self.types.is_empty() {
            &[DataTypeKind::Scalar, DataTypeKind::Vector]
        } else {
            &[
                DataTypeKind::Scalar,
                DataTypeKind::Vector,
                DataTypeKind::User,
            ]
        };

        match allowed.choose(rng).unwrap() {
            DataTypeKind::Scalar => DataType::Scalar(scalar_ty),
            DataTypeKind::Vector => DataType::Vector(rng.gen_range(2..=4), scalar_ty),
            DataTypeKind::User => {
                DataType::User(Rc::new(self.types.keys().choose(rng).cloned().unwrap()))
            }
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&[StructMember]> {
        self.types.get(name).map(|it| it.members.as_slice())
    }

    pub fn into_structs(self) -> Vec<StructDecl> {
        let mut decls = self.types.into_values().collect::<Vec<_>>();
        decls.sort_by(|a, b| a.name.cmp(&b.name));
        decls
    }
}

pub struct FnContext {
    sigs: Vec<Rc<FnSig>>,
    impls: Vec<FnDecl>,
    count: u32,
}

impl FnContext {
    pub fn new(options: Rc<Options>) -> Self {
        FnContext {
            sigs: ast::gen_builtin_fns(options.enabled_fns.iter().map(String::as_str))
                .into_iter()
                .map(Rc::new)
                .collect(),
            impls: vec![],
            count: 0,
        }
    }

    pub fn len(&self) -> u32 {
        self.count
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rc<FnSig>> {
        self.sigs.iter()
    }

    pub fn contains_type(&self, ty: &DataType) -> bool {
        self.iter().any(|sig| matches!(&sig.2, Some(t) if t == ty))
    }

    pub fn select(&self, rng: &mut impl Rng, return_ty: &DataType) -> Option<Rc<FnSig>> {
        self.iter()
            .filter(|sig| matches!(&sig.2, Some(t) if t == return_ty))
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

        self.sigs.push(sig.clone());
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
