use ast::{
    BinOp, BinOpExpr, DataType, ExprNode, FnCallExpr, FnDecl, FnInput, FnOutput, Lit,
    ReturnStatement, ScalarType, VarExpr,
};

pub fn times(name: String, data_type: &DataType) -> FnDecl {
    let condition = match data_type.as_scalar().unwrap() {
        ScalarType::I32 => gen_condition_for_i32(data_type),
        ScalarType::U32 => gen_condition_for_u32(data_type),
        ty => unreachable!("no times wrapper for type {ty}"),
    };

    let happy_path = BinOpExpr::new(
        BinOp::Times,
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
    let a = VarExpr::new("a").into_node(data_type.clone());
    let b = VarExpr::new("b").into_node(data_type.clone());

    let minus_one_special_case = super::componentwise_or(a.clone(), b.clone(), |a, b| {
        BinOpExpr::new(
            BinOp::LogOr,
            BinOpExpr::new(
                BinOp::LogAnd,
                BinOpExpr::new(BinOp::Equal, a.clone(), Lit::I32(-1)),
                BinOpExpr::new(BinOp::Equal, b.clone(), Lit::I32(i32::MIN)),
            ),
            BinOpExpr::new(
                BinOp::LogAnd,
                BinOpExpr::new(BinOp::Equal, a, Lit::I32(i32::MIN)),
                BinOpExpr::new(BinOp::Equal, b, Lit::I32(-1)),
            ),
        )
        .into()
    });

    let overflow_check = super::componentwise_or(a, b, |a, b| {
        BinOpExpr::new(
            BinOp::LogAnd,
            BinOpExpr::new(BinOp::NotEqual, b.clone(), Lit::I32(0)),
            BinOpExpr::new(
                BinOp::LogOr,
                BinOpExpr::new(
                    BinOp::Greater,
                    a.clone(),
                    BinOpExpr::new(BinOp::Divide, Lit::I32(i32::MAX), b.clone()),
                ),
                BinOpExpr::new(
                    BinOp::Less,
                    a,
                    BinOpExpr::new(BinOp::Divide, Lit::I32(i32::MIN), b),
                ),
            ),
        )
        .into()
    });

    BinOpExpr::new(BinOp::LogOr, minus_one_special_case, overflow_check).into()
}

fn gen_condition_for_u32(data_type: &DataType) -> ExprNode {
    super::componentwise_or(
        VarExpr::new("a").into_node(data_type.clone()),
        VarExpr::new("b").into_node(data_type.clone()),
        |a, b| {
            BinOpExpr::new(
                BinOp::LogAnd,
                BinOpExpr::new(BinOp::NotEqual, b.clone(), Lit::U32(0)),
                BinOpExpr::new(
                    BinOp::Greater,
                    a,
                    BinOpExpr::new(BinOp::Divide, Lit::U32(u32::MAX), b),
                ),
            )
            .into()
        },
    )
}
