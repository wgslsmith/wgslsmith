use std::fmt::Display;

use derive_more::Display;

use crate::types::{DataType, ScalarType};

#[derive(Debug, PartialEq)]
pub enum Lit {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Bool(v) => v.fmt(f),
            Lit::I32(v) => v.fmt(f),
            Lit::U32(v) => write!(f, "{v}u"),
            Lit::F32(v) => {
                write!(f, "{v}")?;

                // The default rust formatting for f32 is to not print a decimal point if the
                // number has no fractional component. This is problematic since WGSL will think
                // it's an integer literal, so we manually add a decimal point in that case.
                // TODO: Once naga supports the 'f' suffix for float literals we can switch to that
                // https://github.com/gfx-rs/naga/pull/1863
                if v.fract() == 0.0 {
                    write!(f, ".0")?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum UnOp {
    #[display(fmt = "-")]
    Neg,
    #[display(fmt = "!")]
    Not,
    #[display(fmt = "~")]
    BitNot,
}

impl UnOp {
    /// Determines the return type of a unary operator given its operand type.
    pub fn type_eval(&self, t: &DataType) -> DataType {
        // All unary operators currently produce the same type as the operand type.
        t.clone()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum BinOp {
    #[display(fmt = "+")]
    Plus,
    #[display(fmt = "-")]
    Minus,
    #[display(fmt = "*")]
    Times,
    #[display(fmt = "/")]
    Divide,
    #[display(fmt = "%")]
    Mod,
    #[display(fmt = "&&")]
    LogAnd,
    #[display(fmt = "||")]
    LogOr,
    #[display(fmt = "&")]
    BitAnd,
    #[display(fmt = "|")]
    BitOr,
    #[display(fmt = "^")]
    BitXOr,
    #[display(fmt = "<<")]
    LShift,
    #[display(fmt = ">>")]
    RShift,
    #[display(fmt = "==")]
    Equal,
    #[display(fmt = "!=")]
    NotEqual,
    #[display(fmt = "<")]
    Less,
    #[display(fmt = "<=")]
    LessEqual,
    #[display(fmt = ">")]
    Greater,
    #[display(fmt = ">=")]
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

#[derive(Debug, Display, PartialEq)]
pub enum Postfix {
    #[display(fmt = "[{_0}]")]
    ArrayIndex(Box<ExprNode>),
    #[display(fmt = ".{_0}")]
    Member(String),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Lit(Lit),
    TypeCons(DataType, Vec<ExprNode>),
    Var(String),
    Postfix(Box<ExprNode>, Postfix),
    UnOp(UnOp, Box<ExprNode>),
    BinOp(BinOp, Box<ExprNode>, Box<ExprNode>),
    FnCall(String, Vec<ExprNode>),
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
            Expr::Postfix(primary, pf) => write!(f, "{primary}{pf}"),
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

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{expr}")]
pub struct ExprNode {
    pub data_type: DataType,
    pub expr: Expr,
}
