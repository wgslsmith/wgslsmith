use ast::*;

pub fn index(name: String, data_type: &DataType) -> FnDecl {
    let index = VarExpr::new("index").into_node(data_type.clone());
    let size = VarExpr::new("size").into_node(data_type.clone());

    let (condition, safe_result) = match data_type.as_scalar().unwrap() {
        ScalarType::I32 => {
            let condition = BinOpExpr::new(BinOp::Equal, index.clone(), Lit::I32(i32::MIN));
            let safe_result = Lit::I32(0);
            (condition.into(), safe_result.into())
        }
        ScalarType::U32 => {
            return gen_wrapper(
                name,
                data_type,
                BinOpExpr::new(BinOp::Mod, index, size).into(),
            )
        }
        ty => unreachable!("no divide wrapper for type {ty}"),
    };

    let happy_path = BinOpExpr::new(
        BinOp::Mod,
        FnCallExpr::new("abs", vec![index]).into_node(data_type.clone()),
        size,
    )
    .into();

    gen_wrapper(
        name,
        data_type,
        FnCallExpr::new("select", vec![happy_path, safe_result, condition])
            .into_node(data_type.clone()),
    )
}

fn gen_wrapper(name: String, data_type: &DataType, return_expr: ExprNode) -> FnDecl {
    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput::new("index", data_type.clone()),
            FnInput::new("size", data_type.clone()),
        ],
        output: Some(FnOutput::new(data_type.clone())),
        body: vec![ReturnStatement::new(return_expr).into()],
    }
}
