use ast::{AssignmentLhs, ExprNode, Statement};
use rand::prelude::{SliceRandom, StdRng};

use crate::types::{DataTypeExt, TypeConstraints};

use super::expr::ExprGenerator;
use super::scope::Scope;

pub struct ScopedStmtGenerator<'a> {
    rng: &'a mut StdRng,
    scope: Scope,
}

#[derive(Clone, Copy)]
enum StatementType {
    LetDecl,
    VarDecl,
    Assignment,
    Compound,
    If,
}

impl<'a> ScopedStmtGenerator<'a> {
    pub fn new(rng: &mut StdRng) -> ScopedStmtGenerator {
        ScopedStmtGenerator {
            rng,
            scope: Scope::empty(),
        }
    }

    fn new_scope(&mut self) -> ScopedStmtGenerator {
        ScopedStmtGenerator {
            rng: self.rng,
            scope: self.scope.clone(),
        }
    }

    pub fn gen_stmt(&mut self) -> Statement {
        log::info!("generating statement");

        let mut allowed = vec![
            StatementType::LetDecl,
            StatementType::VarDecl,
            StatementType::Compound,
            StatementType::If,
        ];

        if self.scope.has_vars() {
            allowed.push(StatementType::Assignment);
        }

        match allowed.choose(&mut self.rng).unwrap() {
            StatementType::LetDecl => Statement::LetDecl(
                self.scope.next_name(),
                self.gen_expr(TypeConstraints::Unconstrained()),
            ),
            StatementType::VarDecl => Statement::VarDecl(
                self.scope.next_name(),
                self.gen_expr(TypeConstraints::Unconstrained()),
            ),
            StatementType::Assignment => {
                let (name, data_type) = self.scope.choose_var(&mut self.rng);
                let constraints = data_type.to_constraints();
                Statement::Assignment(
                    AssignmentLhs::Simple(name.clone(), vec![]),
                    self.gen_expr(&constraints),
                )
            }
            StatementType::Compound => Statement::Compound(self.new_scope().gen_block(1)),
            StatementType::If => Statement::If(
                self.gen_expr(TypeConstraints::Bool()),
                self.new_scope().gen_block(1),
            ),
        }
    }

    pub fn gen_block(&mut self, count: u32) -> Vec<Statement> {
        log::info!("generating block of {} statements", count);

        let mut stmts = vec![];

        for _ in 0..count {
            let stmt = self.gen_stmt();

            // If we generated a variable declaration, track it in the environment
            if let Statement::LetDecl(name, expr) = &stmt {
                self.scope.insert_let(name.clone(), expr.data_type.clone());
            } else if let Statement::VarDecl(name, expr) = &stmt {
                self.scope.insert_var(name.clone(), expr.data_type.clone());
            }

            stmts.push(stmt);
        }

        stmts
    }

    fn gen_expr(&mut self, constraints: &TypeConstraints) -> ExprNode {
        ExprGenerator::new(self.rng, &mut self.scope).gen_expr(constraints)
    }
}
