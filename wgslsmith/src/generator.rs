mod expr;
mod scope;
mod stmt;
mod structs;
mod utils;

use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, Attr, AttrList, Expr, ExprNode, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, Lit, Module, Postfix, ShaderStage, Statement, StorageClass, StructDecl,
    StructMember, VarQualifier,
};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::generator::expr::ExprGenerator;
use crate::generator::scope::Scope;
use crate::generator::stmt::ScopedStmtGenerator;
use crate::Options;

use self::scope::{FnRegistry, TypeRegistry};

pub struct Generator {
    rng: StdRng,
    options: Rc<Options>,
}

impl Generator {
    pub fn new(rng: StdRng, options: Rc<Options>) -> Self {
        Generator { rng, options }
    }

    #[tracing::instrument(skip(self))]
    pub fn gen_module(&mut self) -> Module {
        let mut structs = vec![StructDecl {
            name: "Buffer".to_owned(),
            members: vec![StructMember {
                name: "data".to_owned(),
                data_type: DataType::Array(Rc::new(DataType::Scalar(ScalarType::U32))),
            }],
        }];

        let mut ty_reg = TypeRegistry::new();

        for i in self.options.min_structs..=self.options.max_structs {
            let name = format!("Struct_{}", i);
            let decl = structs::gen_struct_decl(&mut self.rng, &ty_reg, &self.options, name);
            ty_reg.insert(decl.clone());
            structs.push(decl);
        }

        let mut fns = FnRegistry::new(self.options.clone());
        let entrypoint = self.gen_entrypoint_function(&Scope::empty(), &mut fns);

        Module {
            structs,
            vars: vec![GlobalVarDecl {
                attrs: AttrList(vec![
                    self.gen_attr(GlobalVarAttr::Group(0)),
                    self.gen_attr(GlobalVarAttr::Binding(0)),
                ]),
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Storage,
                    access_mode: Some(AccessMode::ReadWrite),
                }),
                name: "output".to_owned(),
                data_type: DataType::User(Rc::new("Buffer".to_owned())),
                initializer: None,
            }],
            functions: fns.into_fns(),
            entrypoint,
        }
    }

    fn gen_attr<T>(&mut self, attr: T) -> Attr<T> {
        Attr {
            attr,
            style: *self.options.attribute_style.choose(&mut self.rng).unwrap(),
        }
    }

    #[tracing::instrument(skip(self, scope, fns))]
    fn gen_entrypoint_function(&mut self, scope: &Scope, fns: &mut FnRegistry) -> FnDecl {
        let stmt_count = self.rng.gen_range(5..10);
        let mut gen =
            ScopedStmtGenerator::new(&mut self.rng, scope, None, fns, self.options.clone());
        let mut stmts = gen.gen_block(stmt_count);
        let scope = gen.into_scope();

        stmts.push(Statement::Assignment(
            AssignmentLhs::Simple(
                "output".to_owned(),
                vec![
                    Postfix::Member("data".to_owned()),
                    Postfix::ArrayIndex(Box::new(ExprNode {
                        data_type: DataType::Scalar(ScalarType::U32),
                        expr: Expr::Lit(Lit::UInt(0)),
                    })),
                ],
            ),
            ExprGenerator::new(&mut self.rng, &scope, fns, self.options.clone())
                .gen_expr(&DataType::Scalar(ScalarType::U32)),
        ));

        FnDecl {
            attrs: AttrList(vec![
                self.gen_attr(FnAttr::Stage(ShaderStage::Compute)),
                self.gen_attr(FnAttr::WorkgroupSize(1)),
            ]),
            name: "main".to_owned(),
            inputs: vec![],
            output: None,
            body: stmts,
        }
    }
}
