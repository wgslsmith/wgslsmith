use ast::types::DataType;
use ast::{BinOp, BinOpExpr, FnCallExpr, FnDecl, FnInput, FnOutput, ReturnStatement, VarExpr};

// Clamp is currently unsafe on tint and naga if low > high,
// so we recondition by swapping low and high.
// TODO: Remove once tint and naga fix this
pub fn clamp(name: String, ty: &DataType) -> FnDecl {
    FnDecl {
        attrs: vec![],
        name,
        inputs: vec![
            FnInput::new("e", ty.clone()),
            FnInput::new("low", ty.clone()),
            FnInput::new("high", ty.clone()),
        ],
        output: Some(FnOutput::new(ty.clone())),
        body: vec![ReturnStatement::new(
            FnCallExpr::new(
                "select",
                vec![
                    FnCallExpr::new(
                        "clamp",
                        vec![
                            VarExpr::new("e").into_node(ty.clone()),
                            VarExpr::new("low").into_node(ty.clone()),
                            VarExpr::new("high").into_node(ty.clone()),
                        ],
                    )
                    .into_node(ty.clone()),
                    FnCallExpr::new(
                        "clamp",
                        vec![
                            VarExpr::new("e").into_node(ty.clone()),
                            VarExpr::new("high").into_node(ty.clone()),
                            VarExpr::new("low").into_node(ty.clone()),
                        ],
                    )
                    .into_node(ty.clone()),
                    BinOpExpr::new(
                        BinOp::Greater,
                        VarExpr::new("low").into_node(ty.clone()),
                        VarExpr::new("high").into_node(ty.clone()),
                    )
                    .into(),
                ],
            )
            .into_node(ty.clone()),
        )
        .into()],
    }
}
