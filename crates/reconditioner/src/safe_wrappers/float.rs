use ast::{
    BinOp, DataType, Expr, ExprNode, FnDecl, FnInput, FnOutput, Lit, ReturnStatement, ScalarType,
};

pub fn float(name: String, data_type: &DataType) -> FnDecl {
    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![FnInput {
            attrs: vec![],
            data_type: data_type.clone(),
            name: "v".to_owned(),
        }],
        output: Some(FnOutput {
            attrs: vec![],
            data_type: data_type.clone(),
        }),
        body: vec![ReturnStatement::new(Some(ExprNode {
            data_type: data_type.clone(),
            expr: Expr::FnCall(
                "select".to_owned(),
                vec![
                    ExprNode {
                        data_type: data_type.clone(),
                        expr: Expr::Var("v".to_owned()),
                    },
                    ExprNode {
                        data_type: data_type.clone(),
                        expr: Expr::TypeCons(
                            data_type.clone(),
                            vec![ExprNode {
                                data_type: ScalarType::F32.into(),
                                expr: Expr::Lit(Lit::F32(10.0)),
                            }],
                        ),
                    },
                    ExprNode {
                        data_type: ScalarType::Bool.into(),
                        expr: Expr::BinOp(
                            BinOp::LogOr,
                            Box::new(gen_any(ExprNode {
                                data_type: data_type.map(ScalarType::Bool),
                                expr: Expr::BinOp(
                                    BinOp::Less,
                                    Box::new(ExprNode {
                                        data_type: data_type.clone(),
                                        expr: Expr::FnCall(
                                            "abs".to_owned(),
                                            vec![ExprNode {
                                                data_type: data_type.clone(),
                                                expr: Expr::Var("v".to_owned()),
                                            }],
                                        ),
                                    }),
                                    Box::new(ExprNode {
                                        data_type: data_type.clone(),
                                        expr: Expr::TypeCons(
                                            data_type.clone(),
                                            vec![ExprNode {
                                                data_type: ScalarType::F32.into(),
                                                expr: Expr::Lit(Lit::F32(0.1)),
                                            }],
                                        ),
                                    }),
                                ),
                            })),
                            Box::new(gen_any(ExprNode {
                                data_type: data_type.map(ScalarType::Bool),
                                expr: Expr::BinOp(
                                    BinOp::GreaterEqual,
                                    Box::new(ExprNode {
                                        data_type: data_type.clone(),
                                        expr: Expr::FnCall(
                                            "abs".to_owned(),
                                            vec![ExprNode {
                                                data_type: data_type.clone(),
                                                expr: Expr::Var("v".to_owned()),
                                            }],
                                        ),
                                    }),
                                    Box::new(ExprNode {
                                        data_type: data_type.clone(),
                                        expr: Expr::TypeCons(
                                            data_type.clone(),
                                            vec![ExprNode {
                                                data_type: ScalarType::F32.into(),
                                                expr: Expr::Lit(Lit::F32(16777216.0)),
                                            }],
                                        ),
                                    }),
                                ),
                            })),
                        ),
                    },
                ],
            ),
        }))
        .into()],
    }
}

fn gen_any(expr: ExprNode) -> ExprNode {
    if expr.data_type.is_vector() {
        ExprNode {
            data_type: ScalarType::Bool.into(),
            expr: Expr::FnCall("any".to_owned(), vec![expr]),
        }
    } else {
        expr
    }
}
