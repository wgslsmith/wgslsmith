use rand::Rng;

use crate::ast::{BinOp, Expr, ExprNode, Lit, UnOp};
use crate::types::{DataType, TypeConstraints};

#[derive(Default)]
pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Generator
    }

    pub fn gen_expr(&mut self, constraints: TypeConstraints) -> ExprNode {
        let allowed: &[u32] = if constraints.intersection(TypeConstraints::INT).is_some() {
            // If the constraints allow generating integer expressions then we can generate any
            // expression type
            &[0, 1, 2]
        } else {
            // Otherwise for booleans we can only generate literals currently
            &[0]
        };

        match allowed[rand::thread_rng().gen_range(0..allowed.len())] {
            0 => {
                let (lit, t) = self.gen_lit(constraints);
                ExprNode {
                    data_type: t,
                    expr: Expr::Lit(lit),
                }
            }
            1 => {
                let expr = self.gen_expr(constraints);
                ExprNode {
                    data_type: expr.data_type,
                    expr: Expr::UnOp(self.gen_un_op(), Box::new(expr)),
                }
            }
            2 => {
                let l = self.gen_expr(constraints);
                let r = self.gen_expr(l.data_type.into());
                ExprNode {
                    data_type: l.data_type,
                    expr: Expr::BinOp(self.gen_bin_op(), Box::new(l), Box::new(r)),
                }
            }
            _ => unreachable!(),
        }
    }

    fn gen_lit(&mut self, constraints: TypeConstraints) -> (Lit, DataType) {
        // Select a random concrete type from the constraints
        let t = constraints.select();

        let lit = match t {
            DataType::Bool => Lit::Bool(rand::random()),
            DataType::SInt => Lit::Int(rand::random()),
            DataType::UInt => Lit::UInt(rand::random()),
        };

        (lit, t)
    }

    fn gen_un_op(&mut self) -> UnOp {
        UnOp::Neg
    }

    fn gen_bin_op(&mut self) -> BinOp {
        match rand::thread_rng().gen_range(0..5) {
            0 => BinOp::Plus,
            1 => BinOp::Minus,
            2 => BinOp::Times,
            3 => BinOp::Divide,
            4 => BinOp::Mod,
            _ => unreachable!(),
        }
    }
}
