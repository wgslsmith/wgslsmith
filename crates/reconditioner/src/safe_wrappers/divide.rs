use ast::{
    BinOp, BinOpExpr, DataType, ExprNode, FnCallExpr, FnDecl, FnInput, FnOutput, Lit,
    ReturnStatement, ScalarType, TypeConsExpr, VarExpr,
};

pub fn divide(name: String, data_type: &DataType) -> FnDecl {
    let (condition, safe_divisor) = match data_type.as_scalar().unwrap() {
        ScalarType::I32 => (gen_condition_for_i32(data_type), Lit::I32(2)),
        ScalarType::U32 => (gen_condition_for_u32(data_type), Lit::U32(2)),
        ty => unreachable!("no divide wrapper for type {ty}"),
    };

    let happy_path = BinOpExpr::new(
        BinOp::Divide,
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
    )
    .into();

    let safe_result = BinOpExpr::new(
        BinOp::Divide,
        VarExpr::new("a").into_node(data_type.clone()),
        TypeConsExpr::new(data_type.clone(), vec![safe_divisor.into()]),
    )
    .into();

    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput::new("a", data_type.clone()),
            FnInput::new("b", data_type.clone()),
        ],
        output: Some(FnOutput::new(data_type.clone())),
        body: vec![ReturnStatement::new(
            FnCallExpr::new("select", vec![happy_path, safe_result, condition])
                .into_node(data_type.clone()),
        )
        .into()],
    }
}

fn gen_condition_for_i32(data_type: &DataType) -> ExprNode {
    super::componentwise_or(
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
        |a, b| {
            BinOpExpr::new(
                BinOp::LogOr,
                BinOpExpr::new(
                    BinOp::LogAnd,
                    BinOpExpr::new(BinOp::Equal, a, Lit::I32(i32::MIN)),
                    BinOpExpr::new(BinOp::Equal, b.clone(), Lit::I32(-1)),
                ),
                BinOpExpr::new(BinOp::Equal, b, Lit::I32(0)),
            )
            .into()
        },
    )
}

fn gen_condition_for_u32(data_type: &DataType) -> ExprNode {
    super::componentwise_or(
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
        |_, b| BinOpExpr::new(BinOp::Equal, b, Lit::U32(0)).into(),
    )
}
