use std::fmt::{Display, Write};

#[derive(Debug)]
pub enum Lit {
    Bool(bool),
    Int(i32),
    UInt(u32),
}

#[derive(Debug)]
pub enum UnOp {
    Neg,
}

#[derive(Debug)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
}

#[derive(Debug)]
pub enum Expr {
    Lit(Lit),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Bool(v) => v.fmt(f),
            Lit::Int(v) => v.fmt(f),
            Lit::UInt(v) => v.fmt(f),
        }
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => f.write_char('-'),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Plus => f.write_char('+'),
            BinOp::Minus => f.write_char('-'),
            BinOp::Times => f.write_char('*'),
            BinOp::Divide => f.write_char('/'),
            BinOp::Mod => f.write_char('%'),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(v) => v.fmt(f),
            Expr::UnOp(op, e) => write!(f, "{}({})", op, e),
            Expr::BinOp(op, l, r) => write!(f, "({}){}({})", l, op, r),
        }
    }
}
