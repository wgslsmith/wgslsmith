mod expr;
mod scope;
mod stmt;

use std::sync::Arc;

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
use crate::types::TypeConstraints;

pub struct Generator {
    rng: StdRng,
}

impl Generator {
    pub fn new(rng: StdRng) -> Self {
        Generator { rng }
    }

    pub fn gen_module(&mut self) -> Module {
        log::info!("generating module");

        let stmt_count = self.rng.gen_range(0..50);
        let mut stmts = ScopedStmtGenerator::new(&mut self.rng).gen_block(stmt_count);

        log::info!("generating output assignment");

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
            ExprGenerator::new(&mut self.rng, &mut Scope::empty()).gen_expr(TypeConstraints::U32()),
        ));

        Module {
            structs: vec![StructDecl {
                name: "Buffer".to_owned(),
                members: vec![StructMember {
                    name: "data".to_owned(),
                    data_type: DataType::Array(Arc::new(DataType::Scalar(ScalarType::U32))),
                }],
            }],
            vars: vec![GlobalVarDecl {
                attrs: AttrList(vec![GlobalVarAttr::Group(0), GlobalVarAttr::Binding(0)]),
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Storage,
                    access_mode: Some(AccessMode::ReadWrite),
                }),
                name: "output".to_owned(),
                data_type: DataType::User(Arc::new("Buffer".to_owned())),
                initializer: None,
            }],
            functions: vec![],
            entrypoint: FnDecl {
                attrs: AttrList(vec![
                    FnAttr::Stage(ShaderStage::Compute),
                    FnAttr::WorkgroupSize(1),
                ]),
                name: "main".to_owned(),
                inputs: vec![],
                output: None,
                body: stmts,
            },
        }
    }
}
