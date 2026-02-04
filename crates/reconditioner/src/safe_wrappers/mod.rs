mod dot;
mod float;
mod float_divide;
mod index;
mod modulo;

use ast::{
    BinOp, BinOpExpr, DataType, ExprNode, FnCallExpr, Lit, Postfix, PostfixExpr, ScalarType,
};

pub use dot::dot;
pub use float::float;
pub use float_divide::float_divide;
pub use index::index;
pub use modulo::modulo;

/// Wraps the given expression in a call to `any()` if it is a vector.
///
/// TODO: Get rid of this once naga implements the scalar overload for `any`.
/// https://github.com/gfx-rs/naga/issues/1911
fn any(expr: impl Into<ExprNode>) -> ExprNode {
    let expr = expr.into();
    if expr.data_type.is_vector() {
        FnCallExpr::new("any", vec![expr]).into_node(ScalarType::Bool)
    } else {
        expr
    }
}

fn componentwise_or(
    a: impl Into<ExprNode>,
    b: impl Into<ExprNode>,
    f: impl Fn(ExprNode, ExprNode) -> ExprNode,
) -> ExprNode {
    let a: ExprNode = a.into();
    let b: ExprNode = b.into();

    let n = match &a.data_type {
        DataType::Scalar(_) => return f(a, b),
        DataType::Vector(n, _) => *n,
        _ => unreachable!(),
    };

    let f = |i| {
        f(
            PostfixExpr::new(a.clone(), Postfix::index(Lit::I32(i))).into(),
            PostfixExpr::new(b.clone(), Postfix::index(Lit::I32(i))).into(),
        )
    };

    (1..n).fold(f(0), |expr, i| {
        BinOpExpr::new(BinOp::LogOr, expr, f(i as i32)).into()
    })
}
