use rand::prelude::{IteratorRandom, SliceRandom, StdRng};
use rand::Rng;

use ast::types::{DataType, ScalarType};
use ast::{BinOp, Expr, ExprNode, Lit, Postfix, UnOp};

use super::scope::{FnRegistry, Scope};

pub struct ExprGenerator<'a> {
    rng: &'a mut StdRng,
    fns: &'a mut FnRegistry,
    scope: &'a Scope,
    depth: u32,
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
        fns: &'a mut FnRegistry,
    ) -> ExprGenerator<'a> {
        ExprGenerator {
            rng,
            fns,
            scope,
            depth: 0,
        }
    }

    pub fn gen_expr(&mut self, ty: &DataType) -> ExprNode {
        log::info!("generating expr with {:?}, depth={}", ty, self.depth);

        let mut allowed = vec![];

        match ty {
            DataType::Scalar(_) => {
                allowed.push(ExprType::Lit);
            }
            DataType::Vector(_, _) => {
                allowed.push(ExprType::TypeCons);
            }
            DataType::Array(_) => todo!(),
            DataType::User(_) => todo!(),
        }

        // Use better method for expression complexity
        if self.depth < 5 {
            allowed.push(ExprType::UnOp);

            if matches!(
                ty,
                DataType::Scalar(_) | DataType::Vector(_, ScalarType::I32 | ScalarType::U32)
            ) {
                allowed.push(ExprType::BinOp);
            }

            if self.fns.contains_type(ty) || self.fns.len() < 10 {
                allowed.push(ExprType::FnCall);
            }
        }

        if self.scope.iter_vars().any(|(_, t)| t.can_produce_ty(ty)) {
            allowed.push(ExprType::Var);
        }

        log::info!("allowed constructions: {:?}", allowed);

        match *allowed.choose(&mut self.rng).unwrap() {
            ExprType::Lit => {
                let lit = self.gen_lit(ty);
                ExprNode {
                    data_type: ty.clone(),
                    expr: Expr::Lit(lit),
                }
            }
            ExprType::TypeCons => {
                log::info!("generating type_cons with {:?}", ty);

                let mut args = vec![];

                let (n, t) = match ty {
                    DataType::Scalar(t) => (1, *t),
                    DataType::Vector(n, t) => (*n, *t),
                    _ => todo!(),
                };

                self.depth += 1;

                let arg_ty = DataType::Scalar(t);
                for _ in 0..n {
                    args.push(self.gen_expr(&arg_ty))
                }

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
                log::info!("generating var with {:?}, scope={:?}", ty, self.scope);

                let (name, data_type) = self
                    .scope
                    .iter_vars()
                    .filter(|(_, t)| t.can_produce_ty(ty))
                    .choose(&mut self.rng)
                    .map(|(n, t)| (n, t.clone()))
                    .unwrap();

                let mut expr = ExprNode {
                    data_type: data_type.clone(),
                    expr: Expr::Var(name.to_owned()),
                };

                // If the variable does not directly have the same type as the target, it must
                // be a vector so we need to generate the correct accessor to produce a value of the
                // target type.
                if data_type != *ty {
                    let accessor = super::utils::gen_vector_accessor(self.rng, &data_type, ty);
                    expr = ExprNode {
                        data_type: ty.clone(),
                        expr: Expr::Postfix(Box::new(expr), Postfix::Member(accessor)),
                    }
                }

                expr
            }
            ExprType::FnCall => {
                let func = if self.fns.len() < 10 && self.rng.gen_bool(0.2) {
                    self.fns.gen(self.rng, ty)
                } else {
                    match self.fns.select(&mut self.rng, ty) {
                        Some(v) => v,
                        None => self.fns.gen(self.rng, ty),
                    }
                };

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

    fn gen_lit(&mut self, ty: &DataType) -> Lit {
        log::info!("generating lit with {:?}", ty);

        match ty {
            DataType::Scalar(t) => match t {
                ScalarType::Bool => Lit::Bool(self.rng.gen()),
                ScalarType::I32 => Lit::Int(self.rng.gen()),
                ScalarType::U32 => Lit::UInt(self.rng.gen()),
            },
            _ => unreachable!(),
        }
    }

    fn gen_un_op(&mut self, ty: &DataType) -> UnOp {
        log::info!("generating un_op with {:?}", ty);

        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_) => unreachable!(),
            DataType::User(_) => unreachable!(),
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

    fn gen_bin_op(&mut self, ty: &DataType) -> BinOp {
        log::info!("generating bin_op with {:?}", ty);

        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_) => unreachable!(),
            DataType::User(_) => unreachable!(),
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
            DataType::User(_) => todo!(),
        }
    }
}
