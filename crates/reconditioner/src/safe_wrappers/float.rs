use ast::{
    BinOp, BinOpExpr, DataType, ExprNode, FnCallExpr, FnDecl, FnInput, FnOutput, Lit,
    ReturnStatement, ScalarType, TypeConsExpr, VarExpr,
};

pub fn float(name: String, data_type: &DataType) -> FnDecl {
    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![FnInput::new("v", data_type.clone())],
        output: Some(FnOutput::new(data_type.clone())),
        body: vec![ReturnStatement::new(
            FnCallExpr::new(
                "select".to_owned(),
                vec![
                    VarExpr::new("v").into_node(data_type.clone()),
                    TypeConsExpr::new(data_type.clone(), vec![Lit::F32(10.0).into()]).into(),
                    BinOpExpr::new(
                        BinOp::LogOr,
                        gen_any(BinOpExpr::new(
                            BinOp::Less,
                            FnCallExpr::new(
                                "abs",
                                vec![VarExpr::new("v").into_node(data_type.clone())],
                            )
                            .into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![Lit::F32(0.1).into()]),
                        )),
                        gen_any(BinOpExpr::new(
                            BinOp::GreaterEqual,
                            FnCallExpr::new(
                                "abs",
                                vec![VarExpr::new("v").into_node(data_type.clone())],
                            )
                            .into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![Lit::F32(16777216.0).into()]),
                        )),
                    )
                    .into(),
                ],
            )
            .into_node(data_type.clone()),
        )
        .into()],
    }
}

fn gen_any(expr: impl Into<ExprNode>) -> ExprNode {
    let expr = expr.into();
    if expr.data_type.is_vector() {
        FnCallExpr::new("any", vec![expr]).into_node(ScalarType::Bool)
    } else {
        expr
    }
}
