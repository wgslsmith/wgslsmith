use ast::types::{DataType, ScalarType};
use ast::{BinOp, Expr, ExprNode, FnDecl, FnInput, FnOutput, ReturnStatement};

// Clamp is currently unsafe on tint and naga if low > high,
// so we recondition by swapping low and high.
// TODO: Remove once tint and naga fix this
pub fn clamp(name: String, ty: &DataType) -> FnDecl {
    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput {
                attrs: vec![],
                name: "e".to_owned(),
                data_type: ty.clone(),
            },
            FnInput {
                attrs: vec![],
                name: "low".to_owned(),
                data_type: ty.clone(),
            },
            FnInput {
                attrs: vec![],
                name: "high".to_owned(),
                data_type: ty.clone(),
            },
        ],
        output: Some(FnOutput {
            attrs: vec![],
            data_type: ty.clone(),
        }),
        body: vec![ReturnStatement::new(Some(ExprNode {
            data_type: ty.clone(),
            expr: Expr::FnCall(
                "select".to_owned(),
                vec![
                    ExprNode {
                        data_type: ty.clone(),
                        expr: Expr::FnCall(
                            "clamp".to_owned(),
                            vec![
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("e".to_owned()),
                                },
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("low".to_owned()),
                                },
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("high".to_owned()),
                                },
                            ],
                        ),
                    },
                    ExprNode {
                        data_type: ty.clone(),
                        expr: Expr::FnCall(
                            "clamp".to_owned(),
                            vec![
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("e".to_owned()),
                                },
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("high".to_owned()),
                                },
                                ExprNode {
                                    data_type: ty.clone(),
                                    expr: Expr::Var("low".to_owned()),
                                },
                            ],
                        ),
                    },
                    ExprNode {
                        data_type: ScalarType::Bool.into(),
                        expr: Expr::BinOp(
                            BinOp::Greater,
                            Box::new(ExprNode {
                                data_type: ty.clone(),
                                expr: Expr::Var("low".to_owned()),
                            }),
                            Box::new(ExprNode {
                                data_type: ty.clone(),
                                expr: Expr::Var("high".to_owned()),
                            }),
                        ),
                    },
                ],
            ),
        }))
        .into()],
    }
}
