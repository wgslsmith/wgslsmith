use ast::types::{DataType, MemoryViewType, ScalarType};
use ast::{
    AssignmentLhs, AssignmentOp, AssignmentStatement, Expr, ExprNode, ForLoopHeader, ForLoopInit,
    ForLoopStatement, ForLoopUpdate, IfStatement, LetDeclStatement, LhsExprNode, Lit,
    LoopStatement, ReturnStatement, Statement, StorageClass, SwitchCase, SwitchStatement, UnOp,
    UnOpExpr, VarDeclStatement, VarExpr,
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
        if self.options.enable_pointers && self.scope.has_mutables() && self.rng.gen_bool(0.2) {
            let (ident, ty) = self.scope.choose_mutable(self.rng);
            let initializer =
                UnOpExpr::new(UnOp::AddressOf, VarExpr::new(ident).into_node(ty.clone()));
            LetDeclStatement::new(self.scope.next_name(), initializer).into()
        } else {
            let ty = self.cx.types.select(self.rng);
            LetDeclStatement::new(self.scope.next_name(), self.gen_expr(&ty)).into()
        }
    }

    fn gen_var_stmt(&mut self) -> Statement {
        let ty = self.cx.types.select(self.rng);
        VarDeclStatement::new(self.scope.next_name(), None, Some(self.gen_expr(&ty))).into()
    }

    fn gen_assignment_stmt(&mut self) -> Statement {
        let (name, data_type) = self.scope.choose_mutable(self.rng);

        let data_type = data_type.clone();
        let lhs = match &data_type {
            DataType::Vector(n, ty) if self.rng.gen_bool(0.7) => {
                let accessor =
                    super::utils::gen_vector_accessor(self.rng, *n, &DataType::Scalar(*ty));
                LhsExprNode::member(name.clone(), data_type, accessor)
            }
            DataType::Array(_, _) => LhsExprNode::array_index(
                name.clone(),
                data_type,
                self.gen_expr(&ScalarType::U32.into()),
            ),
            _ => LhsExprNode::name(name.clone(), data_type),
        };

        let rhs = self.gen_expr(lhs.data_type.dereference());

        AssignmentStatement::new(lhs.into(), AssignmentOp::Simple, rhs).into()
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

        IfStatement::new(
            self.gen_expr(&DataType::Scalar(ScalarType::Bool)),
            self.gen_stmt_block(max_count).1,
        )
        .into()
    }

    fn gen_return_stmt(&mut self) -> Statement {
        ReturnStatement::optional(
            self.return_type
                .clone()
                .as_ref()
                .map(|ty| self.gen_expr(ty)),
        )
        .into()
    }

    fn gen_loop_stmt(&mut self) -> Statement {
        let max_count = self
            .rng
            .gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);

        LoopStatement::new(self.gen_stmt_block(max_count).1).into()
    }

    fn gen_switch_stmt(&mut self) -> Statement {
        let selector = self.gen_expr(&DataType::Scalar(ScalarType::I32));
        let case_count: u32 = self.rng.gen_range(0..=4);
        let cases = (0..case_count)
            .map(|_| {
                let block_size = self.rng.gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);
                SwitchCase {
                    selector: ExprNode {
                        data_type: DataType::Scalar(ScalarType::I32),
                        expr: Expr::Lit(Lit::I32(self.gen_i32())),
                    },
                    body: self.gen_stmt_block(block_size).1,
                }
            })
            .collect();

        let default_block_size = self.rng.gen_range(self.options.block_min_stmts..=self.options.block_max_stmts);

        SwitchStatement::new(selector, cases, self.gen_stmt_block(default_block_size).1).into()
    }

    fn gen_for_stmt(&mut self) -> Statement {
        let (_, stmt) = self.with_scope(self.scope.clone(), |this| {
            let (init, update) = if this.rng.gen_bool(0.8) {
                let loop_var = this.scope.next_name();
                let loop_var_type = DataType::Scalar(ScalarType::I32);

                let init_value = if this.rng.gen_bool(0.7) {
                    Some(Lit::I32(this.gen_i32()).into())
                } else {
                    None
                };

                // Specify the type explicitly if we didn't generate an initializer (otherwise let it be
                // inferred).
                let init_type = if init_value.is_none() {
                    Some(loop_var_type.clone())
                } else {
                    None
                };

                let init = ForLoopInit::VarDecl(VarDeclStatement::new(
                    loop_var.clone(),
                    init_type,
                    init_value,
                ));

                this.scope.insert_mutable(
                    loop_var.clone(),
                    DataType::Ref(MemoryViewType::new(
                        loop_var_type.clone(),
                        StorageClass::Function,
                    )),
                );

                let update = if this.rng.gen_bool(0.8) {
                    let assignment_op = if this.rng.gen_bool(0.5) {
                        AssignmentOp::Plus
                    } else {
                        AssignmentOp::Minus
                    };

                    let lhs = AssignmentLhs::name(loop_var, loop_var_type);
                    let rhs = Lit::I32(1);

                    Some(ForLoopUpdate::Assignment(AssignmentStatement::new(
                        lhs,
                        assignment_op,
                        rhs,
                    )))
                } else {
                    None
                };

                (Some(init), update)
            } else {
                (None, None)
            };

            let condition = if this.rng.gen_bool(0.5) {
                Some(this.gen_expr(&DataType::Scalar(ScalarType::Bool)))
            } else {
                None
            };

            let body_size = this
                .rng
                .gen_range(this.options.block_min_stmts..=this.options.block_max_stmts);

            let header = ForLoopHeader {
                init,
                condition,
                update,
            };

            ForLoopStatement::new(header, this.gen_stmt_block(body_size).1)
        });

        stmt.into()
    }

    pub fn gen_stmt_block(&mut self, max_count: u32) -> (Scope, Vec<Statement>) {
        self.with_scope(self.scope.clone(), |this| {
            this.block_depth += 1;

            let prev_block = std::mem::take(&mut this.current_block);

            for _ in 0..max_count {
                let stmt = this.gen_stmt();

                // If we generated a variable declaration, track it in the environment
                if let Statement::LetDecl(stmt) = &stmt {
                    this.scope
                        .insert_readonly(stmt.ident.clone(), stmt.initializer.data_type.clone());
                } else if let Statement::VarDecl(stmt) = &stmt {
                    let mem_view =
                        MemoryViewType::new(stmt.inferred_type().clone(), StorageClass::Function);
                    let data_type = DataType::Ref(mem_view);
                    this.scope.insert_mutable(stmt.ident.clone(), data_type);
                } else if let Statement::Return(_) = &stmt {
                    // Return statement must be the last statement in the block
                    this.block_depth -= 1;
                    return std::mem::replace(&mut this.current_block, prev_block);
                }

                this.current_block.push(stmt);
            }

            this.block_depth -= 1;

            std::mem::replace(&mut this.current_block, prev_block)
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
                    block.push(ReturnStatement::new(this.gen_expr(&return_type)).into())
                });
            }
        }

        block
    }
}
