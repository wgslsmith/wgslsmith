use ast::{BinOp, Expr, Lit, UnOp};
use rand::Rng;

mod ast;

fn main() {
    println!("{}", gen_expr());
}

fn gen_expr() -> Expr {
    match rand::thread_rng().gen_range(0..3) {
        0 => Expr::Lit(gen_lit()),
        1 => Expr::UnOp(gen_un_op(), Box::new(gen_expr())),
        2 => Expr::BinOp(gen_bin_op(), Box::new(gen_expr()), Box::new(gen_expr())),
        _ => unreachable!(),
    }
}

fn gen_lit() -> Lit {
    match rand::thread_rng().gen_range(0..3) {
        0 => Lit::Bool(rand::random()),
        1 => Lit::Int(rand::random()),
        2 => Lit::UInt(rand::random()),
        _ => unreachable!(),
    }
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
