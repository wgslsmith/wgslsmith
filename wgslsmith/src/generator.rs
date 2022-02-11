mod expr;
mod scope;
mod stmt;
mod utils;

use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, AttrList, Expr, ExprNode, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, Lit, Module, Postfix, ShaderStage, Statement, StorageClass, StructDecl,
    StructMember, VarQualifier,
};
use rand::prelude::StdRng;
use rand::Rng;

use crate::generator::expr::ExprGenerator;
use crate::generator::scope::Scope;
use crate::generator::stmt::ScopedStmtGenerator;
use crate::Options;

use self::scope::FnRegistry;

pub struct Generator {
    rng: StdRng,
}

impl Generator {
    pub fn new(rng: StdRng) -> Self {
        Generator { rng }
    }

    #[tracing::instrument(skip(self, options))]
    pub fn gen_module(&mut self, options: Rc<Options>) -> Module {
        let mut fns = FnRegistry::new(options.clone());
        let entrypoint = self.gen_entrypoint_function(&Scope::empty(), &mut fns, options);

        Module {
            structs: vec![StructDecl {
                name: "Buffer".to_owned(),
                members: vec![StructMember {
                    name: "data".to_owned(),
                    data_type: DataType::Array(Rc::new(DataType::Scalar(ScalarType::U32))),
                }],
            }],
            vars: vec![GlobalVarDecl {
                attrs: AttrList(vec![GlobalVarAttr::Group(0), GlobalVarAttr::Binding(0)]),
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

    #[tracing::instrument(skip(self, scope, fns, options))]
    fn gen_entrypoint_function(
        &mut self,
        scope: &Scope,
        fns: &mut FnRegistry,
        options: Rc<Options>,
    ) -> FnDecl {
        let stmt_count = self.rng.gen_range(5..10);
        let mut gen = ScopedStmtGenerator::new(&mut self.rng, scope, None, fns, options.clone());
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
            ExprGenerator::new(&mut self.rng, &scope, fns, options)
                .gen_expr(&DataType::Scalar(ScalarType::U32)),
        ));

        FnDecl {
            attrs: AttrList(vec![
                FnAttr::Stage(ShaderStage::Compute),
                FnAttr::WorkgroupSize(1),
            ]),
            name: "main".to_owned(),
            inputs: vec![],
            output: None,
            body: stmts,
        }
    }
}
