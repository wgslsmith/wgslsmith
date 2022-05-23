use ast::{
    BinOp, BinOpExpr, DataType, ExprNode, FnCallExpr, FnDecl, FnInput, FnOutput, Lit,
    ReturnStatement, ScalarType, VarExpr,
};

pub fn minus(name: String, data_type: &DataType) -> FnDecl {
    let condition = match data_type.as_scalar().unwrap() {
        ScalarType::I32 => gen_condition_for_i32(data_type),
        ScalarType::U32 => gen_condition_for_u32(data_type),
        ty => unreachable!("no minus wrapper for type {ty}"),
    };

    let happy_path = BinOpExpr::new(
        BinOp::Minus,
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
    )
    .into();

    let safe_result = VarExpr::new("a").into_node(data_type.clone());

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
    // (b < 0 && (a > INT_MAX + b)) || (b > 0 && (a < INT_MIN + b))
    BinOpExpr::new(
        BinOp::LogOr,
        super::componentwise_or(
            VarExpr::new("a").into_node(data_type.clone()),
            VarExpr::new("b").into_node(data_type.clone()),
            |a, b| {
                BinOpExpr::new(
                    BinOp::LogAnd,
                    BinOpExpr::new(BinOp::Less, b.clone(), Lit::I32(0)),
                    BinOpExpr::new(
                        BinOp::Greater,
                        a,
                        BinOpExpr::new(BinOp::Plus, Lit::I32(i32::MAX), b),
                    ),
                )
                .into()
            },
        ),
        super::componentwise_or(
            VarExpr::new("a").into_node(data_type.clone()),
            VarExpr::new("b").into_node(data_type.clone()),
            |a, b| {
                BinOpExpr::new(
                    BinOp::LogAnd,
                    BinOpExpr::new(BinOp::Greater, b.clone(), Lit::I32(0)),
                    BinOpExpr::new(
                        BinOp::Less,
                        a,
                        BinOpExpr::new(BinOp::Plus, Lit::I32(i32::MIN), b),
                    ),
                )
                .into()
            },
        ),
    )
    .into()
}

fn gen_condition_for_u32(data_type: &DataType) -> ExprNode {
    // a < b
    BinOpExpr::new(
        BinOp::Less,
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
    )
    .into()
}
