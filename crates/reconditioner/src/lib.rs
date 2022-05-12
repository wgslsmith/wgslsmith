mod safe_wrappers;

use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AssignmentLhs, AssignmentOp, AssignmentStatement, BinOp, Else, Expr, ExprNode, FnDecl, FnInput,
    FnOutput, ForLoopHeader, ForLoopStatement, GlobalVarDecl, IfStatement, LetDeclStatement, Lit,
    LoopStatement, Module, Postfix, ReturnStatement, Statement, StorageClass, SwitchCase,
    SwitchStatement, UnOp, VarDeclStatement, VarQualifier,
};

pub struct ReconditionResult {
    pub ast: Module,
    pub loop_count: u32,
}

#[derive(Hash, PartialEq, Eq)]
enum Wrapper {
    Clamp(DataType),
}

impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, ty) = match self {
            Wrapper::Clamp(ty) => ("CLAMP", ty),
        };

        write!(f, "SAFE_{}_", name)?;

        match ty {
            DataType::Scalar(ty) => write!(f, "{}", ty),
            DataType::Vector(n, ty) => write!(f, "vec{}_{}", n, ty),
            DataType::Array(_, _) => todo!(),
            DataType::Struct(_) => todo!(),
        }
    }
}

pub fn recondition(mut ast: Module) -> Module {
    let mut reconditioner = Reconditioner::default();

    let functions = ast
        .functions
        .into_iter()
        .map(|f| reconditioner.recondition_fn(f))
        .collect::<Vec<_>>();

    let scalar_wrappers = scalar_safe_wrappers()
        .into_iter()
        .filter(|it| reconditioner.arithmetic_wrappers.contains(&it.name));

    let vector_wrappers = vector_safe_wrappers()
        .into_iter()
        .filter(|it| reconditioner.arithmetic_wrappers.contains(&it.name));

    let safe_wrappers = reconditioner.wrappers.iter().map(|it| match it {
        Wrapper::Clamp(ty) => safe_wrappers::clamp(it.to_string(), ty),
    });

    ast.functions = scalar_wrappers
        .chain(vector_wrappers)
        .chain(safe_wrappers)
        .chain(functions)
        .collect();

    if reconditioner.loop_var > 0 {
        ast.vars.push(GlobalVarDecl {
            attrs: vec![],
            data_type: DataType::Array(
                Rc::new(DataType::Scalar(ScalarType::U32)),
                Some(reconditioner.loop_var),
            ),
            name: "LOOP_COUNTERS".into(),
            initializer: None,
            qualifier: Some(VarQualifier {
                storage_class: StorageClass::Private,
                access_mode: None,
            }),
        });
    }

    ast
}

#[derive(Default)]
struct Reconditioner {
    loop_var: u32,
    wrappers: HashSet<Wrapper>,
    arithmetic_wrappers: HashSet<String>,
}

impl Reconditioner {
    fn recondition_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.recondition_stmt(s))
            .collect();
        decl
    }

    fn recondition_else(&mut self, els: Else) -> Else {
        match els {
            Else::If(IfStatement {
                condition,
                body,
                else_,
            }) => Else::If(IfStatement {
                condition: self.recondition_expr(condition),
                body: body.into_iter().map(|s| self.recondition_stmt(s)).collect(),
                else_: else_.map(|els| Box::new(self.recondition_else(*els))),
            }),
            Else::Else(stmts) => Else::Else(
                stmts
                    .into_iter()
                    .map(|s| self.recondition_stmt(s))
                    .collect(),
            ),
        }
    }

    fn recondition_stmt(&mut self, stmt: Statement) -> Statement {
        match stmt {
            Statement::LetDecl(LetDeclStatement { ident, initializer }) => {
                LetDeclStatement::new(ident, self.recondition_expr(initializer)).into()
            }
            Statement::VarDecl(VarDeclStatement {
                ident,
                data_type,
                initializer,
            }) => VarDeclStatement::new(
                ident,
                data_type,
                initializer.map(|e| self.recondition_expr(e)),
            )
            .into(),
            Statement::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                AssignmentStatement::new(lhs, op, self.recondition_expr(rhs)).into()
            }
            Statement::Compound(s) => {
                Statement::Compound(s.into_iter().map(|s| self.recondition_stmt(s)).collect())
            }
            Statement::If(IfStatement {
                condition,
                body,
                else_,
            }) => IfStatement::new(
                self.recondition_expr(condition),
                body.into_iter().map(|s| self.recondition_stmt(s)).collect(),
            )
            .with_else(else_.map(|els| self.recondition_else(*els)))
            .into(),
            Statement::Return(ReturnStatement { value }) => {
                ReturnStatement::new(value.map(|e| self.recondition_expr(e))).into()
            }
            Statement::Loop(LoopStatement { body }) => LoopStatement::new({
                let id = self.loop_var();
                std::iter::once(
                    IfStatement::new(
                        ExprNode {
                            data_type: DataType::Scalar(ScalarType::Bool),
                            expr: Expr::BinOp(
                                BinOp::GreaterEqual,
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Postfix(
                                        Box::new(ExprNode {
                                            data_type: DataType::Array(
                                                Rc::new(DataType::Scalar(ScalarType::U32)),
                                                None,
                                            ),
                                            expr: Expr::Var("LOOP_COUNTERS".to_owned()),
                                        }),
                                        Postfix::ArrayIndex(Box::new(ExprNode {
                                            data_type: DataType::Scalar(ScalarType::U32),
                                            expr: Expr::Lit(Lit::UInt(id)),
                                        })),
                                    ),
                                }),
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Lit(Lit::UInt(1)),
                                }),
                            ),
                        },
                        vec![Statement::Break],
                    )
                    .into(),
                )
                .chain(std::iter::once(
                    AssignmentStatement::new(
                        AssignmentLhs::Simple(
                            "LOOP_COUNTERS".to_owned(),
                            vec![Postfix::ArrayIndex(Box::new(ExprNode {
                                data_type: DataType::Scalar(ScalarType::U32),
                                expr: Expr::Lit(Lit::UInt(id)),
                            }))],
                        ),
                        AssignmentOp::Simple,
                        ExprNode {
                            data_type: DataType::Scalar(ScalarType::U32),
                            expr: Expr::BinOp(
                                BinOp::Plus,
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Postfix(
                                        Box::new(ExprNode {
                                            data_type: DataType::Array(
                                                Rc::new(DataType::Scalar(ScalarType::U32)),
                                                None,
                                            ),
                                            expr: Expr::Var("LOOP_COUNTERS".to_owned()),
                                        }),
                                        Postfix::ArrayIndex(Box::new(ExprNode {
                                            data_type: DataType::Scalar(ScalarType::U32),
                                            expr: Expr::Lit(Lit::UInt(id)),
                                        })),
                                    ),
                                }),
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Lit(Lit::UInt(1)),
                                }),
                            ),
                        },
                    )
                    .into(),
                ))
                .chain(body.into_iter().map(|s| self.recondition_stmt(s)))
                .collect()
            })
            .into(),
            Statement::Break => Statement::Break,
            Statement::Switch(SwitchStatement {
                selector,
                cases,
                default,
            }) => SwitchStatement::new(
                self.recondition_expr(selector),
                cases
                    .into_iter()
                    .map(|SwitchCase { selector, body }| SwitchCase {
                        selector: self.recondition_expr(selector),
                        body: body
                            .into_iter()
                            .map(|it| self.recondition_stmt(it))
                            .collect(),
                    })
                    .collect(),
                default
                    .into_iter()
                    .map(|it| self.recondition_stmt(it))
                    .collect(),
            )
            .into(),
            Statement::ForLoop(ForLoopStatement { header, body }) => {
                let id = self.loop_var();
                let body = std::iter::once(
                    IfStatement::new(
                        ExprNode {
                            data_type: DataType::Scalar(ScalarType::Bool),
                            expr: Expr::BinOp(
                                BinOp::GreaterEqual,
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Postfix(
                                        Box::new(ExprNode {
                                            data_type: DataType::Array(
                                                Rc::new(DataType::Scalar(ScalarType::U32)),
                                                None,
                                            ),
                                            expr: Expr::Var("LOOP_COUNTERS".to_owned()),
                                        }),
                                        Postfix::ArrayIndex(Box::new(ExprNode {
                                            data_type: DataType::Scalar(ScalarType::U32),
                                            expr: Expr::Lit(Lit::UInt(id)),
                                        })),
                                    ),
                                }),
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Lit(Lit::UInt(1)),
                                }),
                            ),
                        },
                        vec![Statement::Break],
                    )
                    .into(),
                )
                .chain(std::iter::once(
                    AssignmentStatement::new(
                        AssignmentLhs::Simple(
                            "LOOP_COUNTERS".to_owned(),
                            vec![Postfix::ArrayIndex(Box::new(ExprNode {
                                data_type: DataType::Scalar(ScalarType::U32),
                                expr: Expr::Lit(Lit::UInt(id)),
                            }))],
                        ),
                        AssignmentOp::Simple,
                        ExprNode {
                            data_type: DataType::Scalar(ScalarType::U32),
                            expr: Expr::BinOp(
                                BinOp::Plus,
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Postfix(
                                        Box::new(ExprNode {
                                            data_type: DataType::Array(
                                                Rc::new(DataType::Scalar(ScalarType::U32)),
                                                None,
                                            ),
                                            expr: Expr::Var("LOOP_COUNTERS".to_owned()),
                                        }),
                                        Postfix::ArrayIndex(Box::new(ExprNode {
                                            data_type: DataType::Scalar(ScalarType::U32),
                                            expr: Expr::Lit(Lit::UInt(id)),
                                        })),
                                    ),
                                }),
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::U32),
                                    expr: Expr::Lit(Lit::UInt(1)),
                                }),
                            ),
                        },
                    )
                    .into(),
                ))
                .chain(body.into_iter().map(|s| self.recondition_stmt(s)))
                .collect();

                ForLoopStatement::new(
                    ForLoopHeader {
                        init: header.init,
                        condition: header.condition.map(|e| self.recondition_expr(e)),
                        update: header.update,
                    },
                    body,
                )
                .into()
            }
        }
    }

    fn recondition_expr(&mut self, expr: ExprNode) -> ExprNode {
        let reconditioned = match expr.expr {
            Expr::TypeCons(ty, args) => Expr::TypeCons(
                ty,
                args.into_iter().map(|e| self.recondition_expr(e)).collect(),
            ),
            Expr::UnOp(op, e) => {
                let e = self.recondition_expr(*e);
                match op {
                    // TODO: Workaround for bug in naga which generates incorrect code for double
                    // negation expression: https://github.com/gfx-rs/naga/issues/1564.
                    // We transform a double negation into a single negation which is multiplied by -1.
                    UnOp::Neg => match &e.expr {
                        Expr::UnOp(UnOp::Neg, _) | Expr::Lit(Lit::Int(i32::MIN..=-1)) => {
                            Expr::BinOp(
                                BinOp::Times,
                                Box::new(ExprNode {
                                    data_type: DataType::Scalar(ScalarType::I32),
                                    expr: Expr::TypeCons(
                                        e.data_type.clone(),
                                        vec![ExprNode {
                                            data_type: DataType::Scalar(ScalarType::I32),
                                            expr: Expr::Lit(Lit::Int(-1)),
                                        }],
                                    ),
                                }),
                                Box::new(e),
                            )
                        }
                        _ => Expr::UnOp(op, Box::new(e)),
                    },
                    _ => Expr::UnOp(op, Box::new(e)),
                }
            }
            Expr::BinOp(op, l, r) => {
                let l = self.recondition_expr(*l);
                let r = self.recondition_expr(*r);
                return self.recondition_bin_op_expr(expr.data_type, op, l, r);
            }
            Expr::FnCall(name, args) => {
                let args: Vec<ExprNode> =
                    args.into_iter().map(|e| self.recondition_expr(e)).collect();

                match name.as_str() {
                    "clamp" => Expr::FnCall(
                        self.safe_wrapper(Wrapper::Clamp(args[0].data_type.clone())),
                        args,
                    ),
                    _ => Expr::FnCall(name, args),
                }
            }
            Expr::Postfix(e, postfix) => {
                let e = Box::new(self.recondition_expr(*e));

                let postfix = match postfix {
                    Postfix::ArrayIndex(index) => {
                        let len_lit = match e.data_type {
                            DataType::Array(_, Some(n)) => Lit::Int(n as i32),
                            DataType::Array(_, None) => {
                                panic!("cannot recondition array access for runtime sized array")
                            }
                            _ => unreachable!(),
                        };

                        let len_lit_expr = ExprNode {
                            data_type: index.data_type.clone(),
                            expr: Expr::Lit(len_lit),
                        };

                        Postfix::ArrayIndex(Box::new(self.recondition_expr(ExprNode {
                            data_type: index.data_type.clone(),
                            expr: Expr::BinOp(BinOp::Mod, index, Box::new(len_lit_expr)),
                        })))
                    }
                    Postfix::Member(n) => Postfix::Member(n),
                };

                Expr::Postfix(e, postfix)
            }
            e => e,
        };

        ExprNode {
            data_type: expr.data_type,
            expr: reconditioned,
        }
    }

    fn recondition_shift_expr(
        &mut self,
        ty: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        ExprNode {
            data_type: ty.clone(),
            expr: Expr::BinOp(
                op,
                Box::new(l),
                Box::new(ExprNode {
                    data_type: ty.clone(),
                    expr: Expr::BinOp(
                        BinOp::Mod,
                        Box::new(r),
                        Box::new(ExprNode {
                            data_type: ty.map(ScalarType::U32),
                            expr: match ty {
                                DataType::Scalar(_) => Expr::Lit(Lit::UInt(32)),
                                DataType::Vector(_, scalar_ty) => Expr::TypeCons(
                                    ty.map(ScalarType::U32),
                                    vec![ExprNode {
                                        data_type: scalar_ty.into(),
                                        expr: Expr::Lit(Lit::UInt(32)),
                                    }],
                                ),
                                _ => unreachable!(),
                            },
                        }),
                    ),
                }),
            ),
        }
    }

    fn recondition_bin_op_expr(
        &mut self,
        data_type: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        if let BinOp::LShift | BinOp::RShift = op {
            return self.recondition_shift_expr(data_type, op, l, r);
        }

        if matches!(
            data_type,
            DataType::Scalar(ScalarType::I32) | DataType::Vector(_, ScalarType::I32)
        ) {
            let name = match op {
                BinOp::Plus => self.arithmetic_wrapper("PLUS", &data_type),
                BinOp::Minus => self.arithmetic_wrapper("MINUS", &data_type),
                BinOp::Times => self.arithmetic_wrapper("TIMES", &data_type),
                BinOp::Divide => self.arithmetic_wrapper("DIVIDE", &data_type),
                BinOp::Mod => self.arithmetic_wrapper("MOD", &data_type),
                op => {
                    return ExprNode {
                        data_type,
                        expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
                    }
                }
            };

            ExprNode {
                data_type,
                expr: Expr::FnCall(name, vec![l, r]),
            }
        } else {
            ExprNode {
                data_type,
                expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
            }
        }
    }

    fn loop_var(&mut self) -> u32 {
        let cur = self.loop_var;
        self.loop_var += 1;
        cur
    }

    fn safe_wrapper(&mut self, wrapper: Wrapper) -> String {
        let ident = wrapper.to_string();
        self.wrappers.insert(wrapper);
        ident
    }

    fn arithmetic_wrapper(&mut self, name: &str, data_type: &DataType) -> String {
        let ident = safe_fn(name, data_type);

        if !self.arithmetic_wrappers.contains(&ident) {
            self.arithmetic_wrappers.insert(ident.clone());

            if let DataType::Vector(_, ty) = data_type {
                self.arithmetic_wrappers
                    .insert(safe_fn(name, &DataType::Scalar(*ty)));
            }
        }

        ident
    }
}

fn safe_fn(name: &str, data_type: &DataType) -> String {
    let mut ident = String::new();

    write!(ident, "SAFE_{}_", name).unwrap();

    match data_type {
        DataType::Scalar(ty) => write!(ident, "{}", ty).unwrap(),
        DataType::Vector(n, ty) => write!(ident, "vec{}_{}", n, ty).unwrap(),
        DataType::Array(_, _) => todo!(),
        DataType::Struct(_) => todo!(),
    }

    ident
}

fn scalar_safe_wrappers() -> Vec<FnDecl> {
    let functions = [
        include_str!("safe_wrappers/plus_i32.wgsl"),
        include_str!("safe_wrappers/minus_i32.wgsl"),
        include_str!("safe_wrappers/times_i32.wgsl"),
        include_str!("safe_wrappers/divide_i32.wgsl"),
        include_str!("safe_wrappers/mod_i32.wgsl"),
    ];

    functions
        .into_iter()
        .map(|it| {
            it.replace("INT_MAX", "2147483647")
                .replace("INT_MIN", "-2147483648")
        })
        .map(|it| parser::parse_fn(&it))
        .collect()
}

/// Generates safe wrapper functions for vectors. These will forward to the correspoding safe scalar
/// wrapper for each vector component.
fn vector_safe_wrappers() -> Vec<FnDecl> {
    let mut fns = vec![];

    for op in ["PLUS", "MINUS", "TIMES", "DIVIDE", "MOD"] {
        for ty in [ScalarType::I32, ScalarType::U32] {
            for n in 2..=4 {
                let vec_ty = DataType::Vector(n, ty);
                fns.push(FnDecl {
                    attrs: vec![],
                    name: safe_fn(op, &vec_ty),
                    inputs: vec![
                        FnInput {
                            attrs: vec![],
                            name: "a".to_owned(),
                            data_type: vec_ty.clone(),
                        },
                        FnInput {
                            attrs: vec![],
                            name: "b".to_owned(),
                            data_type: vec_ty.clone(),
                        },
                    ],
                    output: Some(FnOutput {
                        attrs: vec![],
                        data_type: vec_ty.clone(),
                    }),
                    body: vec![ReturnStatement::new(Some(ExprNode {
                        data_type: vec_ty.clone(),
                        expr: Expr::TypeCons(
                            vec_ty.clone(),
                            (0..n)
                                .map(|i| {
                                    let component = match i {
                                        0 => "x",
                                        1 => "y",
                                        2 => "z",
                                        3 => "w",
                                        _ => unreachable!(),
                                    };

                                    ExprNode {
                                        data_type: DataType::Scalar(ty),
                                        expr: Expr::FnCall(
                                            safe_fn(op, &DataType::Scalar(ty)),
                                            vec![
                                                ExprNode {
                                                    data_type: DataType::Scalar(ty),
                                                    expr: Expr::Postfix(
                                                        Box::new(ExprNode {
                                                            data_type: vec_ty.clone(),
                                                            expr: Expr::Var("a".to_owned()),
                                                        }),
                                                        Postfix::Member(component.to_owned()),
                                                    ),
                                                },
                                                ExprNode {
                                                    data_type: DataType::Scalar(ty),
                                                    expr: Expr::Postfix(
                                                        Box::new(ExprNode {
                                                            data_type: vec_ty.clone(),
                                                            expr: Expr::Var("b".to_owned()),
                                                        }),
                                                        Postfix::Member(component.to_owned()),
                                                    ),
                                                },
                                            ],
                                        ),
                                    }
                                })
                                .collect(),
                        ),
                    }))
                    .into()],
                });
            }
        }
    }

    fns
}
