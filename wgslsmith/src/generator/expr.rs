use std::rc::Rc;

use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use ast::types::{DataType, ScalarType};
use ast::{BinOp, Expr, ExprNode, Lit, Postfix, StructDecl, UnOp};

use crate::generator::cx::FnSig;
use crate::Options;

use super::cx::Context;
use super::fns;
use super::scope::Scope;

pub fn gen_expr(
    rng: &mut StdRng,
    cx: &Context,
    scope: &Scope,
    options: &Options,
    ty: &DataType,
) -> ExprNode {
    ExprGenerator::new(rng, scope, cx, options).gen_expr(ty)
}

struct ExprGenerator<'a> {
    rng: &'a mut StdRng,
    cx: &'a Context,
    scope: &'a Scope,
    depth: u32,
    options: &'a Options,
}

#[derive(Clone, Copy, Debug)]
enum ExprType {
    Lit,
    TypeCons,
    Var,
    UnOp,
    BinOp,
    FnCall,
}

impl<'a> ExprGenerator<'a> {
    pub fn new(
        rng: &'a mut StdRng,
        scope: &'a Scope,
        cx: &'a Context,
        options: &'a Options,
    ) -> ExprGenerator<'a> {
        ExprGenerator {
            rng,
            cx,
            scope,
            depth: 0,
            options,
        }
    }

    #[tracing::instrument(skip(self), fields(self.depth))]
    pub fn gen_expr(&mut self, ty: &DataType) -> ExprNode {
        let mut allowed = vec![];

        match ty {
            DataType::Scalar(_) => {
                allowed.push(ExprType::Lit);
            }
            DataType::Vector(_, _) => {
                allowed.push(ExprType::TypeCons);
            }
            DataType::Array(_) => todo!(),
            DataType::Struct(_) => {
                allowed.push(ExprType::TypeCons);
            }
        }

        // Use better method for expression complexity
        if self.depth < 5 {
            if matches!(ty, DataType::Scalar(_) | DataType::Vector(_, _)) {
                allowed.push(ExprType::UnOp);
            }

            if matches!(
                ty,
                DataType::Scalar(_) | DataType::Vector(_, ScalarType::I32 | ScalarType::U32)
            ) {
                allowed.push(ExprType::BinOp);
            }

            let fns = self.cx.fns.borrow();
            if fns.contains_type(ty) || fns.len() < self.options.max_fns {
                allowed.push(ExprType::FnCall);
            }
        }

        if !self.scope.of_type(ty).is_empty() {
            allowed.push(ExprType::Var);
        }

        tracing::info!("allowed constructions: {:?}", allowed);

        match *allowed.choose(&mut self.rng).unwrap() {
            ExprType::Lit => {
                let lit = self.gen_lit(ty);
                ExprNode {
                    data_type: ty.clone(),
                    expr: Expr::Lit(lit),
                }
            }
            ExprType::TypeCons => {
                tracing::info!("generating type_cons with {:?}", ty);

                self.depth += 1;

                let args = match ty {
                    DataType::Scalar(t) => vec![self.gen_expr(&DataType::Scalar(*t))],
                    DataType::Vector(n, t) => (0..*n)
                        .map(|_| self.gen_expr(&DataType::Scalar(*t)))
                        .collect(),
                    DataType::Array(_) => todo!(),
                    DataType::Struct(decl) => decl
                        .members
                        .iter()
                        .map(|it| self.gen_expr(&it.data_type))
                        .collect(),
                };

                self.depth -= 1;

                ExprNode {
                    data_type: ty.clone(),
                    expr: Expr::TypeCons(ty.clone(), args),
                }
            }
            ExprType::UnOp => {
                self.depth += 1;

                let op = self.gen_un_op(ty);
                let expr = self.gen_expr(ty);

                self.depth -= 1;

                ExprNode {
                    data_type: op.type_eval(&expr.data_type),
                    expr: Expr::UnOp(op, Box::new(expr)),
                }
            }
            ExprType::BinOp => {
                self.depth += 1;

                let op = self.gen_bin_op(ty);
                let l_ty = match op {
                    // These operators work on scalar/vector integers.
                    // The result type depends on the operand type.
                    | BinOp::Plus
                    | BinOp::Minus
                    | BinOp::Times
                    | BinOp::Divide
                    | BinOp::Mod
                    | BinOp::BitXOr
                    | BinOp::LShift
                    | BinOp::RShift => ty.clone(),

                    // These operators work on any scalar/vector.
                    // The result type depends on the operand type.
                    BinOp::BitAnd | BinOp::BitOr => ty.clone(),

                    // These operators only work on scalar bools.
                    BinOp::LogAnd | BinOp::LogOr => ty.clone(),

                    // These operators work on scalar/vector integers.
                    // The number of components in the result type depends on the operands, but the
                    // actual type does not.
                    BinOp::Less | BinOp::LessEqual | BinOp::Greater | BinOp::GreaterEqual => ty
                        .map(
                            [ScalarType::I32, ScalarType::U32]
                                .choose(&mut self.rng)
                                .copied()
                                .unwrap(),
                        ),

                    // These operators work on scalar/vector integers and bools.
                    // The number of components in the result type depends on the operands, but the
                    // actual type does not.
                    BinOp::Equal | BinOp::NotEqual => ty.map(
                        [ScalarType::I32, ScalarType::U32, ScalarType::Bool]
                            .choose(&mut self.rng)
                            .copied()
                            .unwrap(),
                    ),
                };

                let l = self.gen_expr(&l_ty);
                let r_ty = match op {
                    // For shifts, right operand must be u32
                    BinOp::LShift | BinOp::RShift => l.data_type.map(ScalarType::U32),
                    // For everything else right operand must be same type as left
                    _ => l.data_type.clone(),
                };

                let r = self.gen_expr(&r_ty);

                self.depth -= 1;

                ExprNode {
                    data_type: op.type_eval(&l.data_type, &r.data_type),
                    expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
                }
            }
            ExprType::Var => {
                tracing::info!("generating var with {:?}, scope={:?}", ty, self.scope);

                let (name, data_type) = self
                    .scope
                    .of_type(ty)
                    .choose(&mut self.rng)
                    .map(|(n, t)| (n, t.clone()))
                    .unwrap();

                let expr = ExprNode {
                    data_type: data_type.clone(),
                    expr: Expr::Var(name.to_owned()),
                };

                if data_type == *ty {
                    return expr;
                }

                // Variable does not have the same type as the target, so we need to generate an
                // accessor to get an appropriate field
                self.gen_accessor(&data_type, ty, expr)
            }
            ExprType::FnCall => {
                fn maybe_gen_fn(
                    rng: &mut StdRng,
                    cx: &Context,
                    options: &Options,
                    ty: &DataType,
                ) -> Rc<FnSig> {
                    let fns = cx.fns.borrow();

                    // Produce a function call with p=0.8 or p=1 if max functions reached
                    if fns.len() > options.max_fns || rng.gen_bool(0.8) {
                        if let Some(func) = fns.select(rng, ty) {
                            return func;
                        }
                    }

                    drop(fns);

                    // Otherwise generate a new function with the target return type
                    let decl = fns::gen_fn(rng, cx, options, ty);

                    // Add the new function to the context
                    cx.fns.borrow_mut().insert(decl)
                }

                let func = maybe_gen_fn(self.rng, self.cx, self.options, ty);

                let (name, params, return_type) = func.as_ref();
                let return_type = return_type.as_ref().unwrap();

                self.depth += 1;
                let args = params.iter().map(|ty| self.gen_expr(ty)).collect();
                self.depth -= 1;

                ExprNode {
                    data_type: return_type.clone(),
                    expr: Expr::FnCall(name.clone(), args),
                }
            }
        }
    }

    fn gen_accessor(&mut self, ty: &DataType, target: &DataType, expr: ExprNode) -> ExprNode {
        match ty {
            DataType::Scalar(_) => unreachable!(),
            DataType::Vector(n, _) => self.gen_vector_accessor(*n, target, expr),
            DataType::Array(_) => todo!(),
            DataType::Struct(decl) => self.gen_struct_accessor(decl, target, expr),
        }
    }

    fn gen_vector_accessor(&mut self, size: u8, target: &DataType, expr: ExprNode) -> ExprNode {
        let accessor = super::utils::gen_vector_accessor(self.rng, size, target);
        ExprNode {
            data_type: target.clone(),
            expr: Expr::Postfix(Box::new(expr), Postfix::Member(accessor)),
        }
    }

    fn gen_struct_accessor(
        &mut self,
        decl: &StructDecl,
        target: &DataType,
        expr: ExprNode,
    ) -> ExprNode {
        let member = decl.accessors_of(target).choose(self.rng).unwrap();
        let expr = ExprNode {
            data_type: target.clone(),
            expr: Expr::Postfix(Box::new(expr), Postfix::Member(member.name.clone())),
        };

        if member.data_type == *target {
            return expr;
        }

        self.gen_accessor(&member.data_type, target, expr)
    }

    #[tracing::instrument(skip(self))]
    fn gen_lit(&mut self, ty: &DataType) -> Lit {
        tracing::info!("generating lit with {:?}", ty);

        match ty {
            DataType::Scalar(t) => match t {
                ScalarType::Bool => Lit::Bool(self.rng.gen()),
                ScalarType::I32 => Lit::Int(self.rng.gen()),
                ScalarType::U32 => Lit::UInt(self.rng.gen()),
            },
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(skip(self))]
    fn gen_un_op(&mut self, ty: &DataType) -> UnOp {
        tracing::info!("generating un_op with {:?}", ty);

        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_) => unreachable!(),
            DataType::Struct(_) => unreachable!(),
        };

        match scalar_ty {
            ScalarType::Bool => UnOp::Not,
            ScalarType::U32 => UnOp::BitNot,
            ScalarType::I32 => [UnOp::Neg, UnOp::BitNot]
                .choose(&mut self.rng)
                .copied()
                .unwrap(),
        }
    }

    #[tracing::instrument(skip(self))]
    fn gen_bin_op(&mut self, ty: &DataType) -> BinOp {
        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_) => unreachable!(),
            DataType::Struct(_) => unreachable!(),
        };

        let allowed: &[BinOp] = match scalar_ty {
            ScalarType::Bool => &[
                BinOp::Equal,
                BinOp::NotEqual,
                BinOp::Less,
                BinOp::LessEqual,
                BinOp::Greater,
                BinOp::GreaterEqual,
                BinOp::BitAnd,
                BinOp::BitOr,
            ],
            ScalarType::I32 | ScalarType::U32 => &[
                BinOp::Plus,
                BinOp::Minus,
                BinOp::Times,
                BinOp::Divide,
                BinOp::Mod,
                BinOp::BitAnd,
                BinOp::BitOr,
                BinOp::BitXOr,
                BinOp::LShift,
                BinOp::RShift,
            ],
        };

        let mut allowed = allowed.to_vec();

        if let DataType::Scalar(ScalarType::Bool) = ty {
            allowed.extend_from_slice(&[BinOp::LogAnd, BinOp::LogOr]);
        }

        *allowed.choose(&mut self.rng).unwrap()
    }
}

trait DataTypeExt {
    fn can_produce_ty(&self, ty: &DataType) -> bool;
}

impl DataTypeExt for DataType {
    /// Returns true if `ty` can be produced from `self`.
    ///
    /// This is the case if `ty` and `self` are the same, or if `ty` can be produced by performing
    /// an array index or member access on `self`, e.g. `some_vec.x`.
    fn can_produce_ty(&self, ty: &DataType) -> bool {
        if self == ty {
            return true;
        }

        match ty {
            DataType::Scalar(s) | DataType::Vector(_, s) => {
                matches!(self, DataType::Vector(_, t) if s == t)
            }
            DataType::Array(_) => todo!(),
            DataType::Struct(_) => todo!(),
        }
    }
}
