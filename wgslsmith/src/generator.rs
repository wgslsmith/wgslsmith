mod expr;
mod scope;
mod stmt;

use ast::types::{DataType, ScalarType};
use ast::{AssignmentLhs, Expr, ExprNode, FnAttr, FnDecl, Lit, Module, ShaderStage, Statement};
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
            AssignmentLhs::ArrayIndex {
                name: "output.data".to_owned(),
                index: ExprNode {
                    data_type: DataType::Scalar(ScalarType::U32),
                    expr: Expr::Lit(Lit::UInt(0)),
                },
            },
            ExprGenerator::new(&mut self.rng, &mut Scope::empty()).gen_expr(TypeConstraints::U32()),
        ));

        Module {
            entrypoint: FnDecl {
                attrs: vec![
                    FnAttr::Stage(ShaderStage::Compute),
                    FnAttr::WorkgroupSize(1),
                ],
                name: "main".to_owned(),
                inputs: vec![],
                output: None,
                body: stmts,
            },
        }
    }
}
