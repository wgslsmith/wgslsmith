use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{AttrList, FnDecl, FnInput, FnOutput, Statement};
use rand::prelude::{IteratorRandom, SliceRandom, StdRng};
use rand::Rng;
use rpds::Vector;

use crate::Options;

use super::expr::ExprGenerator;
use super::stmt::ScopedStmtGenerator;

pub type FnSig = (String, Vec<DataType>, Option<DataType>);

pub struct FnRegistry {
    sigs: Vec<Rc<FnSig>>,
    impls: Vec<FnDecl>,
    count: u32,
}

impl FnRegistry {
    pub fn new(options: &Options) -> Self {
        FnRegistry {
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

    #[tracing::instrument(skip(self, rng))]
    pub fn gen(&mut self, rng: &mut StdRng, return_ty: &DataType) -> Rc<FnSig> {
        let name = self.next_fn();

        let arg_count = rng.gen_range(0..5);
        let args = (0..arg_count)
            .map(|i| FnInput {
                attrs: AttrList(vec![]),
                name: format!("arg_{}", i),
                data_type: self.gen_ty(rng),
            })
            .collect();

        let stmt_count = rng.gen_range(5..10);
        // TODO: Global scope should be passed here to allow access to global variables
        let mut gen = ScopedStmtGenerator::new(rng, &Scope::empty(), Some(return_ty.clone()), self);
        let mut stmts = gen.gen_block(stmt_count);
        let scope = gen.into_scope();

        if !matches!(stmts.last(), Some(Statement::Return(_))) {
            stmts.push(Statement::Return(Some(
                ExprGenerator::new(rng, &scope, self).gen_expr(return_ty),
            )))
        }

        let decl = FnDecl {
            attrs: AttrList(vec![]),
            name,
            inputs: args,
            output: Some(FnOutput {
                attrs: AttrList(vec![]),
                data_type: return_ty.clone(),
            }),
            body: stmts,
        };

        self.insert(decl)
    }

    fn gen_ty(&self, rng: &mut impl Rng) -> DataType {
        let scalar_ty = [ScalarType::I32, ScalarType::U32, ScalarType::Bool]
            .choose(rng)
            .copied()
            .unwrap();

        match rng.gen_range(0..2) {
            0 => DataType::Scalar(scalar_ty),
            1 => DataType::Vector(rng.gen_range(2..=4), scalar_ty),
            _ => unreachable!(),
        }
    }

    fn next_fn(&mut self) -> String {
        self.count += 1;
        format!("func_{}", self.count)
    }

    pub fn into_fns(self) -> Vec<FnDecl> {
        self.impls
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    next_name: u32,
    consts: Vector<(String, DataType)>,
    vars: Vector<(String, DataType)>,
}

impl Scope {
    pub fn empty() -> Scope {
        Scope {
            next_name: 0,
            consts: Vector::new(),
            vars: Vector::new(),
        }
    }

    pub fn has_vars(&self) -> bool {
        !self.vars.is_empty()
    }

    pub fn iter_vars(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.consts
            .iter()
            .chain(self.vars.iter())
            .map(|(n, t)| (n, t))
    }

    pub fn choose_var(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.vars.iter().choose(rng).map(|(n, t)| (n, t)).unwrap()
    }

    pub fn insert_let(&mut self, name: String, data_type: DataType) {
        self.consts.push_back_mut((name, data_type));
    }

    pub fn insert_var(&mut self, name: String, data_type: DataType) {
        self.vars.push_back_mut((name, data_type));
    }

    pub fn next_var(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }
}
