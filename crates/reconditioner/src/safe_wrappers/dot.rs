use ast::{
    DataType, FnCallExpr, FnDecl, FnInput, FnOutput, Lit, ReturnStatement, ScalarType,
    TypeConsExpr, VarExpr,
};

pub fn dot(name: String, data_type: &DataType) -> FnDecl {
    let (n, scalar) = match data_type {
        DataType::Vector(n, scalar) => (n, scalar),
        _ => unreachable!("dot can only be applied to vectors"),
    };

    // Dot product currently has undefined behaviour due to integer overflow in tint and naga.
    // The values below are computed by taking floor(sqrt(u32::MAX / n)) for a vector of size
    // n, and similarly for i32. By ensuring that each component of the vector is within these
    // bounds, the operation will never overflow.
    // TODO: Remove this wrapper once tint and naga have their own overflow checks.
    let (min, max) = match (n, scalar) {
        (2, ScalarType::U32) => (Lit::U32(0), Lit::U32(46340)),
        (3, ScalarType::U32) => (Lit::U32(0), Lit::U32(37837)),
        (4, ScalarType::U32) => (Lit::U32(0), Lit::U32(32767)),
        (2, ScalarType::I32) => (Lit::I32(-32767), Lit::I32(32767)),
        (3, ScalarType::I32) => (Lit::I32(-26754), Lit::I32(26754)),
        (4, ScalarType::I32) => (Lit::I32(-23170), Lit::I32(23170)),
        _ => unreachable!(),
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
                    FnCallExpr::new(
                        "clamp",
                        vec![
                            VarExpr::new("a").into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![min.into()]).into(),
                            TypeConsExpr::new(data_type.clone(), vec![max.into()]).into(),
                        ],
                    )
                    .into_node(data_type.clone()),
                    FnCallExpr::new(
                        "clamp",
                        vec![
                            VarExpr::new("b").into_node(data_type.clone()),
                            TypeConsExpr::new(data_type.clone(), vec![min.into()]).into(),
                            TypeConsExpr::new(data_type.clone(), vec![max.into()]).into(),
                        ],
                    )
                    .into_node(data_type.clone()),
                ],
            )
            .into_node(scalar),
        )
        .into()],
    }
}
