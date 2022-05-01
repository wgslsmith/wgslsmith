use ast::types::{DataType, ScalarType};
use ast::{
    AssignmentLhs, AssignmentOp, Expr, ExprNode, ForInit, ForUpdate, Lit, Postfix, Statement,
};
use rand::prelude::SliceRandom;
use rand::Rng;

use super::scope::Scope;

#[derive(Clone, Copy)]
enum StatementType {
    LetDecl,
    VarDecl,
    Assignment,
    Compound,
    If,
    Return,
    Loop,
    Switch,
    ForLoop,
}

impl<'a> super::Generator<'a> {
    pub fn gen_stmt(&mut self) -> Statement {
        let mut allowed = vec![
            StatementType::LetDecl,
            StatementType::VarDecl,
            StatementType::Return,
        ];

        if self.scope.has_mutables() {
            allowed.push(StatementType::Assignment);
        }

        if self.block_depth < self.options.max_block_depth {
            allowed.extend_from_slice(&[
                StatementType::Compound,
                StatementType::If,
                StatementType::Loop,
                StatementType::Switch,
                StatementType::ForLoop,
            ]);
        }

        let weights = |t: &StatementType| match t {
            StatementType::LetDecl => 10,
            StatementType::VarDecl => 10,
            StatementType::Assignment => 10,
            StatementType::Compound => 1,
            StatementType::If => 5,
            StatementType::Return => 1,
            StatementType::Loop => 5,
            StatementType::Switch => 5,
            StatementType::ForLoop => 5,
        };

        match allowed.choose_weighted(self.rng, weights).unwrap() {
            StatementType::LetDecl => self.gen_let_stmt(),
            StatementType::VarDecl => self.gen_var_stmt(),
            StatementType::Assignment => self.gen_assignment_stmt(),
            StatementType::Compound => self.gen_compound_stmt(),
            StatementType::If => self.gen_if_stmt(),
            StatementType::Return => self.gen_return_stmt(),
            StatementType::Loop => self.gen_loop_stmt(),
            StatementType::Switch => self.gen_switch_stmt(),
            StatementType::ForLoop => self.gen_for_stmt(),
        }
    }

    fn gen_let_stmt(&mut self) -> Statement {
        let ty = self.cx.types.borrow().select(self.rng);
        Statement::LetDecl(self.scope.next_name(), self.gen_expr(&ty))
    }

    fn gen_var_stmt(&mut self) -> Statement {
        let ty = self.cx.types.borrow().select(self.rng);
        Statement::VarDecl(self.scope.next_name(), self.gen_expr(&ty))
    }

    fn gen_assignment_stmt(&mut self) -> Statement {
        let (name, data_type) = self.scope.choose_mutable(self.rng);

        let data_type = data_type.clone();
        let (lhs, data_type) = match &data_type {
            DataType::Vector(n, ty) if self.rng.gen_bool(0.7) => {
                let accessor =
                    super::utils::gen_vector_accessor(self.rng, *n, &DataType::Scalar(*ty));

                let lhs = AssignmentLhs::Simple(name.clone(), vec![Postfix::Member(accessor)]);

                (lhs, DataType::Scalar(*ty))
            }
            _ => (AssignmentLhs::Simple(name.clone(), vec![]), data_type),
        };

        Statement::Assignment(lhs, AssignmentOp::Simple, self.gen_expr(&data_type))
    }

    fn gen_compound_stmt(&mut self) -> Statement {
        let max_count = self
            .rng
            .gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);
        Statement::Compound(self.gen_stmt_block(max_count).1)
    }

    fn gen_if_stmt(&mut self) -> Statement {
        let max_count = self
            .rng
            .gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);

        Statement::If(
            self.gen_expr(&DataType::Scalar(ScalarType::Bool)),
            self.gen_stmt_block(max_count).1,
            None,
        )
    }

    fn gen_return_stmt(&mut self) -> Statement {
        Statement::Return(
            self.return_type
                .clone()
                .as_ref()
                .map(|ty| self.gen_expr(ty)),
        )
    }

    fn gen_loop_stmt(&mut self) -> Statement {
        let max_count = self
            .rng
            .gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);

        Statement::Loop(self.gen_stmt_block(max_count).1)
    }

    fn gen_switch_stmt(&mut self) -> Statement {
        let selector = self.gen_expr(&DataType::Scalar(ScalarType::I32));
        let case_count: u32 = self.rng.gen_range(0..=4);
        let cases = (0..case_count)
            .map(|_| {
                let block_size = self.rng.gen_range(0..self.options.block_max_stmts);
                (
                    ExprNode {
                        data_type: DataType::Scalar(ScalarType::I32),
                        expr: Expr::Lit(Lit::Int(self.rng.gen())),
                    },
                    self.gen_stmt_block(block_size).1,
                )
            })
            .collect();

        let default_block_size = self.rng.gen_range(0..self.options.block_max_stmts);

        Statement::Switch(selector, cases, self.gen_stmt_block(default_block_size).1)
    }

    fn gen_for_stmt(&mut self) -> Statement {
        let mut scope = self.scope.clone();

        let (init, update) = if self.rng.gen_bool(0.8) {
            let loop_var = scope.next_name();

            let init = ForInit {
                name: loop_var.clone(),
                value: if self.rng.gen_bool(0.7) {
                    scope.insert_var(loop_var.clone(), DataType::Scalar(ScalarType::I32));
                    Some(ExprNode {
                        data_type: DataType::Scalar(ScalarType::I32),
                        expr: Expr::Lit(Lit::Int(self.rng.gen())),
                    })
                } else {
                    None
                },
            };

            let update = if self.rng.gen_bool(0.8) {
                Some(if self.rng.gen_bool(0.5) {
                    ForUpdate::Increment(loop_var)
                } else {
                    ForUpdate::Decrement(loop_var)
                })
            } else {
                None
            };

            (Some(init), update)
        } else {
            (None, None)
        };

        let cond = if self.rng.gen_bool(0.5) {
            Some(self.gen_expr(&DataType::Scalar(ScalarType::Bool)))
        } else {
            None
        };

        let body_size = self
            .rng
            .gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);

        Statement::ForLoop(init, cond, update, self.gen_stmt_block(body_size).1)
    }

    pub fn gen_stmt_block(&mut self, max_count: u32) -> (Scope, Vec<Statement>) {
        self.with_scope(self.scope.clone(), |this| {
            this.block_depth += 1;

            let mut stmts = vec![];

            for _ in 0..max_count {
                let stmt = this.gen_stmt();

                // If we generated a variable declaration, track it in the environment
                if let Statement::LetDecl(name, expr) = &stmt {
                    this.scope.insert_let(name.clone(), expr.data_type.clone());
                } else if let Statement::VarDecl(name, expr) = &stmt {
                    this.scope.insert_var(name.clone(), expr.data_type.clone());
                } else if let Statement::Return(_) = &stmt {
                    // Return statement must be the last statement in the block
                    this.block_depth -= 1;
                    return stmts;
                }

                stmts.push(stmt);
            }

            this.block_depth -= 1;

            stmts
        })
    }

    pub fn gen_stmt_block_with_return(
        &mut self,
        max_count: u32,
        return_type: Option<DataType>,
    ) -> Vec<Statement> {
        let saved_return_type = std::mem::replace(&mut self.return_type, return_type.clone());
        let (scope, mut block) = self.gen_stmt_block(max_count);
        self.return_type = saved_return_type;

        if let Some(return_type) = return_type {
            if !matches!(block.last(), Some(Statement::Return(_))) {
                self.with_scope(scope, |this| {
                    block.push(Statement::Return(Some(this.gen_expr(&return_type))))
                });
            }
        }

        block
    }
}
