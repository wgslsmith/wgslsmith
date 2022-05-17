use std::fmt::Display;
use std::rc::Rc;

use derive_more::{Display, From};

use crate::types::{DataType, ScalarType};

#[derive(Debug, PartialEq)]
pub enum Lit {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
}

impl Lit {
    pub fn data_type(&self) -> DataType {
        match self {
            Lit::Bool(_) => ScalarType::Bool.into(),
            Lit::I32(_) => ScalarType::I32.into(),
            Lit::U32(_) => ScalarType::U32.into(),
            Lit::F32(_) => ScalarType::F32.into(),
        }
    }
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

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{data_type}({})", "crate::FmtArgs(args)")]
pub struct TypeConsExpr {
    pub data_type: DataType,
    pub args: Vec<ExprNode>,
}

impl TypeConsExpr {
    pub fn new(data_type: DataType, args: Vec<ExprNode>) -> Self {
        Self { data_type, args }
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
    #[display(fmt = "&")]
    AddressOf,
    #[display(fmt = "*")]
    Indirection,
}

impl UnOp {
    /// Determines the return type of a unary operator given its operand type.
    pub fn type_eval(&self, t: &DataType) -> DataType {
        match self {
            UnOp::Neg | UnOp::Not | UnOp::BitNot => t.clone(),
            UnOp::AddressOf => DataType::Ptr(Rc::new(t.clone())),
            UnOp::Indirection => t.dereference().unwrap().clone(),
        }
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
    Index(Box<ExprNode>),
    #[display(fmt = ".{_0}")]
    Member(String),
}

impl Postfix {
    pub fn index(expr: impl Into<ExprNode>) -> Postfix {
        Postfix::Index(Box::new(expr.into()))
    }

    pub fn member(member: impl Into<String>) -> Postfix {
        Postfix::Member(member.into())
    }
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{ident}")]
pub struct VarExpr {
    pub ident: String,
}

impl VarExpr {
    pub fn new(ident: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
        }
    }

    pub fn into_node(self, data_type: DataType) -> ExprNode {
        ExprNode {
            data_type,
            expr: self.into(),
        }
    }
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "({inner}){postfix}")]
pub struct PostfixExpr {
    pub inner: Box<ExprNode>,
    pub postfix: Postfix,
}

impl PostfixExpr {
    pub fn new(inner: impl Into<ExprNode>, postfix: Postfix) -> Self {
        Self {
            inner: Box::new(inner.into()),
            postfix,
        }
    }
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{op}({inner})")]
pub struct UnOpExpr {
    pub op: UnOp,
    pub inner: Box<ExprNode>,
}

impl UnOpExpr {
    pub fn new(op: UnOp, inner: impl Into<ExprNode>) -> Self {
        Self {
            op,
            inner: Box::new(inner.into()),
        }
    }
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "({left}) {op} ({right})")]
pub struct BinOpExpr {
    pub op: BinOp,
    pub left: Box<ExprNode>,
    pub right: Box<ExprNode>,
}

impl BinOpExpr {
    pub fn new(op: BinOp, left: impl Into<ExprNode>, right: impl Into<ExprNode>) -> Self {
        Self {
            op,
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
    }
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{ident}({})", "crate::FmtArgs(args)")]
pub struct FnCallExpr {
    pub ident: String,
    pub args: Vec<ExprNode>,
}

impl FnCallExpr {
    pub fn new(ident: impl Into<String>, args: Vec<ExprNode>) -> Self {
        Self {
            ident: ident.into(),
            args,
        }
    }

    pub fn into_node(self, return_type: impl Into<DataType>) -> ExprNode {
        ExprNode {
            data_type: return_type.into(),
            expr: self.into(),
        }
    }
}

#[derive(Debug, Display, PartialEq, From)]
pub enum Expr {
    Lit(Lit),
    TypeCons(TypeConsExpr),
    Var(VarExpr),
    Postfix(PostfixExpr),
    UnOp(UnOpExpr),
    BinOp(BinOpExpr),
    FnCall(FnCallExpr),
}

#[derive(Debug, Display, PartialEq)]
#[display(fmt = "{expr}")]
pub struct ExprNode {
    pub data_type: DataType,
    pub expr: Expr,
}

impl From<Lit> for ExprNode {
    fn from(lit: Lit) -> Self {
        ExprNode {
            data_type: lit.data_type(),
            expr: lit.into(),
        }
    }
}

impl From<TypeConsExpr> for ExprNode {
    fn from(expr: TypeConsExpr) -> Self {
        ExprNode {
            data_type: expr.data_type.clone(),
            expr: expr.into(),
        }
    }
}

impl From<PostfixExpr> for ExprNode {
    fn from(expr: PostfixExpr) -> Self {
        let data_type = match &expr.postfix {
            Postfix::Index(_) => match &expr.inner.data_type {
                DataType::Vector(_, t) => DataType::Scalar(*t),
                DataType::Array(t, _) => (**t).clone(),
                ty => panic!("index operator cannot be applied to type `{ty}`"),
            },
            Postfix::Member(ident) => match &expr.inner.data_type {
                DataType::Struct(decl) => decl.member_type(ident).unwrap().clone(),
                DataType::Vector(_, t) => {
                    if ident.len() == 1 {
                        DataType::Scalar(*t)
                    } else {
                        DataType::Vector(ident.len() as u8, *t)
                    }
                }
                ty => panic!("member access operator cannot be applied to type `{ty}`"),
            },
        };

        ExprNode {
            data_type,
            expr: expr.into(),
        }
    }
}

impl From<UnOpExpr> for ExprNode {
    fn from(expr: UnOpExpr) -> Self {
        ExprNode {
            data_type: expr.op.type_eval(&expr.inner.data_type),
            expr: expr.into(),
        }
    }
}

impl From<BinOpExpr> for ExprNode {
    fn from(expr: BinOpExpr) -> Self {
        ExprNode {
            data_type: expr
                .op
                .type_eval(&expr.left.data_type, &expr.right.data_type),
            expr: expr.into(),
        }
    }
}
