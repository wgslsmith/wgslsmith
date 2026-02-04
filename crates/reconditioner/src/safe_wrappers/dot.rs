use ast::{DataType, FnCallExpr, FnDecl, FnInput, FnOutput, ReturnStatement, VarExpr};

// TODO: Concretize dot and remove this wrapper
pub fn dot(name: String, data_type: &DataType) -> FnDecl {
    let scalar = match data_type {
        DataType::Vector(_n, scalar) => scalar,
        _ => unreachable!("dot can only be applied to vectors"),
    };

    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput::new("a", data_type.clone()),
            FnInput::new("b", data_type.clone()),
        ],
        output: Some(FnOutput::new(scalar)),
        body: vec![ReturnStatement::new(
            FnCallExpr::new(
                "dot",
                vec![
                    VarExpr::new("a").into_node(data_type.clone()),
                    VarExpr::new("b").into_node(data_type.clone()),
                ],
            )
            .into_node(scalar),
        )
        .into()],
    }
}
