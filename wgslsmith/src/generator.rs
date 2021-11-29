use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

use crate::ast::{
    AssignmentLhs, BinOp, Expr, ExprNode, FnAttr, FnDecl, Lit, Module, ShaderStage, Statement, UnOp,
};
use crate::types::{DataType, TypeConstraints};

#[derive(Default)]
pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Generator
    }

    pub fn gen_module(&mut self) -> Module {
        Module {
            entrypoint: FnDecl {
                attrs: vec![
                    FnAttr::Stage(ShaderStage::Compute),
                    FnAttr::WorkgroupSize(1),
                ],
                name: "main".to_owned(),
                inputs: vec![],
                output: None,
                body: vec![Statement::Assignment(
                    AssignmentLhs::ArrayIndex {
                        name: "output.data".to_owned(),
                        index: ExprNode {
                            data_type: DataType::UInt,
                            expr: Expr::Lit(Lit::UInt(0)),
                        },
                    },
                    self.gen_expr(TypeConstraints::UINT),
                )],
            },
        }
    }

    pub fn gen_expr(&mut self, constraints: TypeConstraints) -> ExprNode {
        let allowed: &[u32] = if constraints.contains(TypeConstraints::SINT) {
            // If the constraints allow generating signed integer expressions then we can generate any
            // expression type
            &[0, 1, 2]
        } else if constraints.contains(TypeConstraints::UINT) {
            // For unsigned integers we can't generate UnOp since it's currently only Neg
            &[0, 2]
        } else {
            // Otherwise for booleans we can only generate literals currently
            &[0]
        };

        match allowed[self.rand_range(0..allowed.len())] {
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
            DataType::Bool => Lit::Bool(self.rand()),
            DataType::SInt => Lit::Int(self.rand()),
            DataType::UInt => Lit::UInt(self.rand()),
        };

        (lit, t)
    }

    fn gen_un_op(&mut self) -> UnOp {
        UnOp::Neg
    }

    fn gen_bin_op(&mut self) -> BinOp {
        match self.rand_range(0..5) {
            0 => BinOp::Plus,
            1 => BinOp::Minus,
            2 => BinOp::Times,
            3 => BinOp::Divide,
            4 => BinOp::Mod,
            _ => unreachable!(),
        }
    }

    fn rand<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        rand::random()
    }

    fn rand_range<T: SampleUniform, R: SampleRange<T>>(&mut self, r: R) -> T {
        rand::thread_rng().gen_range(r)
    }
}
