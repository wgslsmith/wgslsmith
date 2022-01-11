use ast::types::{DataType, ScalarType};
use ast::{AssignmentLhs, ExprNode, Statement};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

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

        let weights = |t: &StatementType| {
            if let StatementType::Compound = t {
                1
            } else {
                2
            }
        };

        match allowed.choose_weighted(&mut self.rng, weights).unwrap() {
            StatementType::LetDecl => {
                let ty = self.gen_ty();
                Statement::LetDecl(self.scope.next_name(), self.gen_expr(&ty))
            }
            StatementType::VarDecl => {
                let ty = self.gen_ty();
                Statement::VarDecl(self.scope.next_name(), self.gen_expr(&ty))
            }
            StatementType::Assignment => {
                let (name, data_type) = self.scope.choose_var(&mut self.rng);
                let data_type = data_type.clone();
                Statement::Assignment(
                    AssignmentLhs::Simple(name.clone(), vec![]),
                    self.gen_expr(&data_type),
                )
            }
            StatementType::Compound => Statement::Compound(self.new_scope().gen_block(1)),
            StatementType::If => Statement::If(
                self.gen_expr(&DataType::Scalar(ScalarType::Bool)),
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

    fn gen_ty(&mut self) -> DataType {
        let scalar_ty = [ScalarType::I32, ScalarType::U32, ScalarType::Bool]
            .choose(&mut self.rng)
            .copied()
            .unwrap();

        match self.rng.gen_range(0..2) {
            0 => DataType::Scalar(scalar_ty),
            1 => DataType::Vector(self.rng.gen_range(2..=4), scalar_ty),
            _ => unreachable!(),
        }
    }

    fn gen_expr(&mut self, ty: &DataType) -> ExprNode {
        ExprGenerator::new(self.rng, &self.scope).gen_expr(ty)
    }

    pub fn into_scope(self) -> Scope {
        self.scope
    }
}
