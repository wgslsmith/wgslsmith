use std::iter;

use rand::Rng;
use wgslsmith::ast::{BinOp, Expr, ExprNode, Lit, UnOp};
use wgslsmith::types::{DataType, TypeConstraints};

fn main() {
    println!("{}", gen_expr(TypeConstraints::INT));
}

fn gen_expr(constraints: TypeConstraints) -> ExprNode {
    let allowed: &[u32] = if constraints.intersection(TypeConstraints::INT).is_some() {
        // If the constraints allow generating integer expressions then we can generate any
        // expression type
        &[0, 1, 2]
    } else {
        // Otherwise for booleans we can only generate literals currently
        &[0]
    };

    match allowed[rand::thread_rng().gen_range(0..allowed.len())] {
        0 => {
            let (lit, t) = gen_lit(constraints);
            ExprNode {
                data_type: t,
                expr: Expr::Lit(lit),
            }
        }
        1 => {
            let expr = gen_expr(constraints);
            ExprNode {
                data_type: expr.data_type,
                expr: Expr::UnOp(gen_un_op(), Box::new(expr)),
            }
        }
        2 => {
            let l = gen_expr(constraints);
            let r = gen_expr(TypeConstraints::any_of(iter::once(l.data_type)));
            ExprNode {
                data_type: l.data_type,
                expr: Expr::BinOp(gen_bin_op(), Box::new(l), Box::new(r)),
            }
        }
        _ => unreachable!(),
    }
}

fn gen_lit(constraints: TypeConstraints) -> (Lit, DataType) {
    // Select a random concrete type from the constraints
    let t = constraints.select();

    let lit = match t {
        DataType::Bool => Lit::Bool(rand::random()),
        DataType::SInt => Lit::Int(rand::random()),
        DataType::UInt => Lit::UInt(rand::random()),
    };

    (lit, t)
}

fn gen_un_op() -> UnOp {
    UnOp::Neg
}

fn gen_bin_op() -> BinOp {
    match rand::thread_rng().gen_range(0..5) {
        0 => BinOp::Plus,
        1 => BinOp::Minus,
        2 => BinOp::Times,
        3 => BinOp::Divide,
        4 => BinOp::Mod,
        _ => unreachable!(),
    }
}
