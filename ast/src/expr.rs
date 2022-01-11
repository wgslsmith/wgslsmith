use std::fmt::{Display, Write};

use crate::types::{DataType, ScalarType};

#[derive(Debug, PartialEq, Eq)]
pub enum Lit {
    Bool(bool),
    Int(i32),
    UInt(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

impl UnOp {
    /// Determines the return type of a unary operator given its operand type.
    pub fn type_eval(&self, t: &DataType) -> DataType {
        // All unary operators currently produce the same type as the operand type.
        t.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    LogAnd,
    LogOr,
    BitAnd,
    BitOr,
    BitXOr,
    LShift,
    RShift,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl BinOp {
    /// Determines the return type of a binary operator given its operand types.
    pub fn type_eval(&self, l: &DataType, #[allow(unused)] r: &DataType) -> DataType {
        match self {
            // These operators produce the same result type as the first operand.
            | BinOp::Plus
            | BinOp::Minus
            | BinOp::Times
            | BinOp::Divide
            | BinOp::Mod
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXOr
            | BinOp::LShift
            | BinOp::RShift => l.clone(),

            // These operators always produce scalar bools.
            BinOp::LogAnd | BinOp::LogOr => DataType::Scalar(ScalarType::Bool),

            // These operators produce a scalar/vector bool with the same number of components
            // as the operands (though the operands may have a different scalar type).
            | BinOp::Less
            | BinOp::LessEqual
            | BinOp::Greater
            | BinOp::GreaterEqual
            | BinOp::Equal
            | BinOp::NotEqual => l.map(ScalarType::Bool),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Postfix {
    ArrayIndex(Box<ExprNode>),
    Member(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Lit(Lit),
    TypeCons(DataType, Vec<ExprNode>),
    Var(String),
    Postfix(Box<ExprNode>, Postfix),
    UnOp(UnOp, Box<ExprNode>),
    BinOp(BinOp, Box<ExprNode>, Box<ExprNode>),
    FnCall(String, Vec<ExprNode>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExprNode {
    pub data_type: DataType,
    pub expr: Expr,
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Bool(v) => v.fmt(f),
            Lit::Int(v) => v.fmt(f),
            Lit::UInt(v) => write!(f, "{}u", v),
        }
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => f.write_char('-'),
            UnOp::Not => f.write_char('!'),
            UnOp::BitNot => f.write_char('~'),
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
            BinOp::LogAnd => f.write_str("&&"),
            BinOp::LogOr => f.write_str("||"),
            BinOp::BitAnd => f.write_char('&'),
            BinOp::BitOr => f.write_char('|'),
            BinOp::BitXOr => f.write_char('^'),
            BinOp::LShift => f.write_str("<<"),
            BinOp::RShift => f.write_str(">>"),
            BinOp::Equal => f.write_str("=="),
            BinOp::NotEqual => f.write_str("!="),
            BinOp::Less => f.write_char('<'),
            BinOp::LessEqual => f.write_str("<="),
            BinOp::Greater => f.write_char('>'),
            BinOp::GreaterEqual => f.write_str(">="),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(v) => v.fmt(f),
            Expr::TypeCons(t, args) => {
                write!(f, "{}(", t)?;

                for (i, e) in args.iter().enumerate() {
                    e.fmt(f)?;
                    if i != args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                write!(f, ")")
            }
            Expr::Var(name) => name.fmt(f),
            Expr::UnOp(op, e) => write!(f, "{}({})", op, e),
            Expr::BinOp(op, l, r) => write!(f, "({}) {} ({})", l, op, r),
            Expr::Postfix(primary, pf) => match pf {
                Postfix::ArrayIndex(index) => write!(f, "{}[{}]", primary, index),
                Postfix::Member(name) => write!(f, "{}.{}", primary, name),
            },
            Expr::FnCall(name, args) => {
                write!(f, "{}(", name)?;

                for (i, e) in args.iter().enumerate() {
                    e.fmt(f)?;
                    if i != args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                write!(f, ")")
            }
        }
    }
}

impl Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expr.fmt(f)
    }
}
