use ast::*;

pub fn float_divide(name: String, data_type: &DataType) -> FnDecl {
    let condition = gen_condition(data_type);

    let correct_res = TypeConsExpr::new(data_type.clone(), vec![Lit::F32(42.0).into()]).into();
    let incorrect_res = TypeConsExpr::new(data_type.clone(), vec![Lit::F32(-123.0).into()]).into();

    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput::new("a", data_type.clone()),
            FnInput::new("b", data_type.clone()),
        ],
        output: Some(FnOutput::new(data_type.clone())),
        body: vec![ReturnStatement::new(
            FnCallExpr::new("select", vec![correct_res, incorrect_res, condition])
                .into_node(data_type.clone()),
        )
        .into()],
    }
}

fn gen_condition(data_type: &DataType) -> ExprNode {
    super::componentwise_or(
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
        |a, b| {
            BinOpExpr::new(
                BinOp::Greater,
                FnCallExpr::new(
                    "abs",
                    vec![BinOpExpr::new(BinOp::Divide, a.clone(), b).into()],
                )
                .into_node(data_type.clone()),
                FnCallExpr::new("abs", vec![a]).into_node(data_type.clone()),
            )
            .into()
        },
    )
}
