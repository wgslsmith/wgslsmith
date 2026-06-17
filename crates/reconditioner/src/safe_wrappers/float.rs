use ast::{
    BinOp, BinOpExpr, DataType, FnCallExpr, FnDecl, FnInput, FnOutput, Lit, ReturnStatement,
    ScalarType, TypeConsExpr, VarExpr,
};

pub fn float(name: String, data_type: &DataType) -> FnDecl {
    let (default, small, large) = match data_type.as_scalar() {
        Some(ScalarType::F16) => (
            Lit::F16(half::f16::from_f32(10.0)),
            Lit::F16(half::f16::from_f32(0.1)),
            Lit::F16(half::f16::MAX),
        ),
        _ => (Lit::F32(10.0), Lit::F32(0.1), Lit::F32(16777216.0)),
    };
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
                    TypeConsExpr::new(data_type.clone(), vec![default.into()]).into(),
                    BinOpExpr::new(
                        BinOp::LogOr,
                        super::any(BinOpExpr::new(
                            BinOp::Less,
                            FnCallExpr::new(
                                "abs",
                                vec![VarExpr::new("v").into_node(data_type.clone())],
                            )
                            .into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![small.into()]),
                        )),
                        super::any(BinOpExpr::new(
                            BinOp::GreaterEqual,
                            FnCallExpr::new(
                                "abs",
                                vec![VarExpr::new("v").into_node(data_type.clone())],
                            )
                            .into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![large.into()]),
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
