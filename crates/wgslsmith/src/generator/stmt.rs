use ast::types::{DataType, ScalarType};
use ast::{AssignmentLhs, Postfix, Statement};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::Options;

use super::cx::Context;
use super::expr::gen_expr;
use super::scope::Scope;

pub struct BlockContext {
    depth: u32,
    return_type: Option<DataType>,
}

impl BlockContext {
    pub fn new(return_type: Option<DataType>) -> BlockContext {
        BlockContext {
            depth: 0,
            return_type,
        }
    }

    fn nested(&self) -> BlockContext {
        BlockContext {
            depth: self.depth + 1,
            return_type: self.return_type.clone(),
        }
    }
}

pub fn gen_block_with_return(
    rng: &mut StdRng,
    cx: &Context,
    scope: &Scope,
    return_ty: Option<DataType>,
    options: &Options,
    max_count: u32,
) -> Vec<Statement> {
    let (scope, mut block) = gen_block(
        rng,
        cx,
        scope,
        &BlockContext::new(return_ty.clone()),
        options,
        max_count,
    );

    if let Some(return_ty) = return_ty {
        if !matches!(block.last(), Some(Statement::Return(_))) {
            block.push(Statement::Return(Some(gen_expr(
                rng, cx, &scope, options, &return_ty,
            ))))
        }
    }

    block
}

// #[tracing::instrument(skip(self))]
pub fn gen_block(
    rng: &mut StdRng,
    cx: &Context,
    scope: &Scope,
    block_cx: &BlockContext,
    options: &Options,
    max_count: u32,
) -> (Scope, Vec<Statement>) {
    let mut scope = scope.clone();
    let mut stmts = vec![];

    for _ in 0..max_count {
        let stmt = gen_stmt(rng, cx, &mut scope, block_cx, options);

        // If we generated a variable declaration, track it in the environment
        if let Statement::LetDecl(name, expr) = &stmt {
            scope.insert_let(name.clone(), expr.data_type.clone());
        } else if let Statement::VarDecl(name, expr) = &stmt {
            scope.insert_var(name.clone(), expr.data_type.clone());
        } else if let Statement::Return(_) = &stmt {
            // Return statement must be the last statement in the block
            return (scope, stmts);
        }

        stmts.push(stmt);
    }

    (scope, stmts)
}

#[derive(Clone, Copy)]
enum StatementType {
    LetDecl,
    VarDecl,
    Assignment,
    Compound,
    If,
    Return,
    Loop,
}

pub fn gen_stmt(
    rng: &mut StdRng,
    cx: &Context,
    scope: &mut Scope,
    block_cx: &BlockContext,
    options: &Options,
) -> Statement {
    let types = cx.types.borrow();

    let mut allowed = vec![
        StatementType::LetDecl,
        StatementType::VarDecl,
        StatementType::Return,
    ];

    if scope.has_mutables() {
        allowed.push(StatementType::Assignment);
    }

    if block_cx.depth < options.max_block_depth {
        allowed.extend_from_slice(&[
            StatementType::Compound,
            StatementType::If,
            StatementType::Loop,
        ]);
    }

    let weights = |t: &StatementType| match t {
        StatementType::LetDecl => 10,
        StatementType::VarDecl => 10,
        StatementType::Assignment => 10,
        StatementType::Compound => 1,
        StatementType::If => 10,
        StatementType::Return => 1,
        StatementType::Loop => 5,
    };

    match allowed.choose_weighted(rng, weights).unwrap() {
        StatementType::LetDecl => {
            let ty = types.select(rng);
            Statement::LetDecl(scope.next_name(), gen_expr(rng, cx, scope, options, &ty))
        }
        StatementType::VarDecl => {
            let ty = types.select(rng);
            Statement::VarDecl(scope.next_name(), gen_expr(rng, cx, scope, options, &ty))
        }
        StatementType::Assignment => {
            let (name, data_type) = scope.choose_mutable(rng);

            let data_type = data_type.clone();
            let (lhs, data_type) = match &data_type {
                DataType::Vector(n, ty) if rng.gen_bool(0.7) => {
                    let accessor =
                        super::utils::gen_vector_accessor(rng, *n, &DataType::Scalar(*ty));

                    let lhs = AssignmentLhs::Simple(name.clone(), vec![Postfix::Member(accessor)]);

                    (lhs, DataType::Scalar(*ty))
                }
                _ => (AssignmentLhs::Simple(name.clone(), vec![]), data_type),
            };

            Statement::Assignment(lhs, gen_expr(rng, cx, scope, options, &data_type))
        }
        StatementType::Compound => {
            let max_count = rng.gen_range(options.block_min_stmts..=options.block_max_stmts);
            Statement::Compound(gen_block(rng, cx, scope, &block_cx.nested(), options, max_count).1)
        }
        StatementType::If => {
            let max_count = rng.gen_range(options.block_min_stmts..=options.block_max_stmts);
            Statement::If(
                gen_expr(rng, cx, scope, options, &DataType::Scalar(ScalarType::Bool)),
                gen_block(rng, cx, scope, &block_cx.nested(), options, max_count).1,
            )
        }
        StatementType::Return => Statement::Return(
            block_cx
                .return_type
                .clone()
                .as_ref()
                .map(|ty| gen_expr(rng, cx, scope, options, ty)),
        ),
        StatementType::Loop => {
            let max_count = rng.gen_range(options.block_min_stmts..=options.block_max_stmts);
            Statement::Loop(gen_block(rng, cx, scope, &block_cx.nested(), options, max_count).1)
        }
    }
}
