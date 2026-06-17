use ast::*;

pub fn float_divide(name: String, data_type: &DataType) -> FnDecl {
    let condition = gen_condition(data_type);

    let (correct, incorrect) = match data_type.as_scalar() {
        Some(ScalarType::F16) => (
            Lit::F16(half::f16::from_f32(42.0)),
            Lit::F16(half::f16::from_f32(-123.0)),
        ),
        _ => (Lit::F32(42.0), Lit::F32(-123.0)),
    };

    let correct_res = TypeConsExpr::new(data_type.clone(), vec![correct.into()]).into();
    let incorrect_res = TypeConsExpr::new(data_type.clone(), vec![incorrect.into()]).into();

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
            // Case 1: Detect (0, 0)
            let (zero, max_val) = match data_type.as_scalar() {
                Some(ScalarType::F16) => {
                    (Lit::F16(half::f16::from_f32(0.0)), Lit::F16(half::f16::MAX))
                }
                _ => (Lit::F32(0.0), Lit::F32(3.40282e38)),
            };
            let a_eq_0 = BinOpExpr::new(BinOp::Equal, a.clone(), zero);
            let b_eq_0 = BinOpExpr::new(BinOp::Equal, b.clone(), zero);

            let zero_div_zero = BinOpExpr::new(BinOp::LogAnd, a_eq_0, b_eq_0);

            // Case 2: Detect (+-Inf, +-Inf)
            // This is slightly below MAX but it's ok for now
            let a_abs = FnCallExpr::new("abs", vec![a]).into_node(data_type.clone());
            let b_abs = FnCallExpr::new("abs", vec![b]).into_node(data_type.clone());

            let a_is_inf = BinOpExpr::new(BinOp::Greater, a_abs, max_val);
            let b_is_inf = BinOpExpr::new(BinOp::Greater, b_abs, max_val);

            let inf_div_inf = BinOpExpr::new(BinOp::LogAnd, a_is_inf, b_is_inf);

            BinOpExpr::new(BinOp::LogOr, zero_div_zero, inf_div_inf).into()
        },
    )
}
