use indexmap::IndexMap;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::distributions::Standard;
use rand::prelude::{Distribution, IteratorRandom, SliceRandom, StdRng};
use rand::Rng;

use crate::ast::{
    AssignmentLhs, BinOp, Expr, ExprNode, FnAttr, FnDecl, Lit, Module, ShaderStage, Statement, UnOp,
};
use crate::types::{DataType, TypeConstraints};

pub struct Generator {
    rng: StdRng,
    next_var: u32,
    expression_depth: u32,
    variables: IndexMap<String, DataType>,
    variable_types: Option<TypeConstraints>,
}

impl Generator {
    pub fn new(rng: StdRng) -> Self {
        Generator {
            rng,
            next_var: 0,
            expression_depth: 0,
            variables: IndexMap::new(),
            variable_types: None,
        }
    }

    pub fn gen_module(&mut self) -> Module {
        let stmt_count: u32 = self.rand_range(0..10);
        let mut stmts = vec![];

        for _ in 0..stmt_count {
            let stmt = self.gen_stmt();

            // If we generated a variable declaration, track it in the environment
            if let Statement::VarDecl(name, expr) = &stmt {
                self.variables.insert(name.to_owned(), expr.data_type);
                self.variable_types = Some(
                    self.variable_types
                        .clone()
                        .unwrap_or_else(|| expr.data_type.into())
                        .union(&expr.data_type.into()),
                );
            }

            stmts.push(stmt);
        }

        stmts.push(Statement::Assignment(
            AssignmentLhs::ArrayIndex {
                name: "output.data".to_owned(),
                index: ExprNode {
                    data_type: DataType::UInt,
                    expr: Expr::Lit(Lit::UInt(0)),
                },
            },
            self.gen_expr(TypeConstraints::UInt()),
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

    pub fn gen_stmt(&mut self) -> Statement {
        Statement::VarDecl(
            self.next_var(),
            self.gen_expr(TypeConstraints::Unconstrained()),
        )
    }

    fn next_var(&mut self) -> String {
        let next = self.next_var;
        self.next_var += 1;
        format!("var_{}", next)
    }

    pub fn gen_expr(&mut self, constraints: &TypeConstraints) -> ExprNode {
        let mut allowed = vec![0];

        if self.expression_depth < 10 {
            if constraints.intersects(TypeConstraints::SInt()) {
                allowed.push(1);
            }

            if constraints.intersects(TypeConstraints::Int()) {
                allowed.push(2);
            }

            if matches!(self.variable_types.as_ref(), Some(t) if t.intersects(constraints)) {
                allowed.push(3);
            }
        }

        match allowed.choose(&mut self.rng).unwrap() {
            0 => {
                let (lit, t) = self.gen_lit(constraints);
                ExprNode {
                    data_type: t,
                    expr: Expr::Lit(lit),
                }
            }
            1 => {
                self.expression_depth += 1;

                let op = self.gen_un_op(constraints);
                let constraints = match op {
                    UnOp::Neg => TypeConstraints::SInt(),
                    UnOp::Not => TypeConstraints::Bool(),
                };

                let expr = self.gen_expr(constraints);

                self.expression_depth -= 1;

                ExprNode {
                    data_type: expr.data_type,
                    expr: Expr::UnOp(op, Box::new(expr)),
                }
            }
            2 => {
                self.expression_depth += 1;

                let op = self.gen_bin_op(constraints);
                let constraints = match op {
                    BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Divide | BinOp::Mod => {
                        constraints.intersection(TypeConstraints::Int()).unwrap()
                    }
                    BinOp::LogAnd | BinOp::LogOr => {
                        constraints.intersection(TypeConstraints::Bool()).unwrap()
                    }
                    _ => unreachable!(),
                };

                let l = self.gen_expr(&constraints);
                let r = self.gen_expr(&l.data_type.into());

                self.expression_depth -= 1;

                ExprNode {
                    data_type: l.data_type,
                    expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
                }
            }
            3 => {
                let (name, &data_type) = self
                    .variables
                    .iter()
                    .filter(|(_, t)| constraints.intersects(&(*t).into()))
                    .choose(&mut self.rng)
                    .unwrap();

                ExprNode {
                    data_type,
                    expr: Expr::Var(name.to_owned()),
                }
            }
            _ => unreachable!(),
        }
    }

    fn gen_lit(&mut self, constraints: &TypeConstraints) -> (Lit, DataType) {
        // Select a random concrete type from the constraints
        let t = constraints.select(&mut self.rng);

        let lit = match t {
            DataType::Bool => Lit::Bool(self.rand()),
            DataType::SInt => Lit::Int(self.rand()),
            DataType::UInt => Lit::UInt(self.rand()),
        };

        (lit, t)
    }

    fn gen_un_op(&mut self, constraints: &TypeConstraints) -> UnOp {
        let mut allowed = vec![];

        if constraints.intersects(TypeConstraints::SInt()) {
            allowed.push(0);
        }

        if constraints.intersects(TypeConstraints::Bool()) {
            allowed.push(1);
        }

        match allowed.choose(&mut self.rng).unwrap() {
            0 => UnOp::Neg,
            1 => UnOp::Not,
            _ => unreachable!(),
        }
    }

    fn gen_bin_op(&mut self, constraints: &TypeConstraints) -> BinOp {
        let mut allowed = vec![];

        if constraints.intersects(TypeConstraints::Int()) {
            allowed.extend_from_slice(&[0, 1, 2, 3, 4]);
        }

        if constraints.intersects(TypeConstraints::Bool()) {
            // TODO: Non short-circuiting logical & and | are currently broken in naga
            // https://github.com/gfx-rs/naga/issues/1574
            allowed.extend_from_slice(&[5, 6]);
        }

        match allowed.choose(&mut self.rng).unwrap() {
            0 => BinOp::Plus,
            1 => BinOp::Minus,
            2 => BinOp::Times,
            3 => BinOp::Divide,
            4 => BinOp::Mod,
            5 => BinOp::LogAnd,
            6 => BinOp::LogOr,
            // 7 => BinOp::BitAnd,
            // 8 => BinOp::BitOr,
            _ => unreachable!(),
        }
    }

    fn rand<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    fn rand_range<T: SampleUniform, R: SampleRange<T>>(&mut self, r: R) -> T {
        self.rng.gen_range(r)
    }
}
