use std::fmt::Display;

use derive_more::{Display, From};

use crate::types::{DataType, ScalarType};

#[derive(Clone, Copy, Debug, PartialEq)]
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
            Lit::I32(v) => {
                if *v == i32::MIN {
                    write!(f, "i32({v})")?;
                }
                else {
                    write!(f, "{v}i")?;
                }

                Ok(())
            },
            Lit::U32(v) => write!(f, "{v}u"),
            Lit::F32(v) => {
                write!(f, "{v}f")?;

                // The default rust formatting for f32 is to not print a decimal point if the
                // number has no fractional component. This is problematic since WGSL will think
                // it's an integer literal, so we manually add a decimal point in that case.
                // TODO: Once naga supports the 'f' suffix for float literals we can switch to that
                // https://github.com/gfx-rs/naga/pull/1863

                //TODO: It appears this issue was resolved last year, plus naga is out of 
                // sync with tint on other representations (i32()). But must update naga version
               

                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
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
    Deref,
}

impl UnOp {
    /// Determines the return type of a unary operator given its operand type.
    pub fn type_eval(&self, ty: &DataType) -> DataType {
        match self {
            UnOp::Neg | UnOp::Not | UnOp::BitNot => {
                if let DataType::Ref(view) = ty {
                    self.type_eval(&view.inner)
                } else {
                    ty.clone()
                }
            }
            UnOp::AddressOf => DataType::Ptr(
                ty.as_memory_view()
                    .expect("target of address-of must be a reference")
                    .clone(),
            ),
            UnOp::Deref => DataType::Ref(
                ty.as_memory_view()
                    .expect("expression being dereferenced must be a pointer")
                    .clone(),
            ),
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
    pub fn type_eval(&self, left: &DataType, #[allow(unused)] right: &DataType) -> DataType {
        let left = if let DataType::Ref(view) = left {
            view.inner.as_ref()
        } else {
            left
        };

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
            | BinOp::RShift => left.clone(),

            // These operators always produce scalar bools.
            BinOp::LogAnd | BinOp::LogOr => DataType::Scalar(ScalarType::Bool),

            // These operators produce a scalar/vector bool with the same number of components
            // as the operands (though the operands may have a different scalar type).
            | BinOp::Less
            | BinOp::LessEqual
            | BinOp::Greater
            | BinOp::GreaterEqual
            | BinOp::Equal
            | BinOp::NotEqual => left.map(ScalarType::Bool),
        }
    }

    /// Returns the precendence level of `self`.
    pub fn precedence(&self) -> u32 {
        match self {
            BinOp::BitAnd => 1,
            BinOp::BitOr => 1,
            BinOp::BitXOr => 1,
            BinOp::LogOr => 2,
            BinOp::LogAnd => 3,
            BinOp::Equal => 4,
            BinOp::NotEqual => 4,
            BinOp::Less => 4,
            BinOp::LessEqual => 4,
            BinOp::Greater => 4,
            BinOp::GreaterEqual => 4,
            BinOp::LShift => 5,
            BinOp::RShift => 5,
            BinOp::Plus => 6,
            BinOp::Minus => 6,
            BinOp::Times => 7,
            BinOp::Divide => 7,
            BinOp::Mod => 7,
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
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

    pub fn type_eval(&self, ty: &DataType) -> DataType {
        if let DataType::Ref(view) = ty {
            return DataType::Ref(view.clone_with_type(self.type_eval(&view.inner)));
        }

        match self {
            Postfix::Index(_) => match ty {
                DataType::Vector(_, t) => DataType::Scalar(*t),
                DataType::Array(t, _) => (**t).clone(),
                ty => panic!("index operator cannot be applied to type `{ty}`"),
            },
            Postfix::Member(ident) => match ty {
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
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

impl Display for PostfixExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let PostfixExpr { inner, postfix } = self;
        if matches!(inner.expr, Expr::UnOp(_) | Expr::BinOp(_)) {
            write!(f, "({inner}){postfix}")
        } else {
            write!(f, "{inner}{postfix}")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl Display for UnOpExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UnOpExpr { op, inner } = self;
        if matches!(op, UnOp::Neg) && matches!(inner.expr, Expr::Lit(Lit::I32(0))) {
            return write!(f, "{inner}");
        }
        if matches!(inner.expr, Expr::UnOp(_) | Expr::BinOp(_))
            || matches!(inner.expr, Expr::Lit(Lit::I32(v)) if v < 0)
            || matches!(inner.expr, Expr::Lit(Lit::F32(v)) if v < 0.0)
        {
            write!(f, "{op}({inner})")
        } else {
            write!(f, "{op}{inner}")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl Display for BinOpExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BinOpExpr { op, left, right } = self;

        if matches!(left.expr, Expr::BinOp(_)) {
            write!(f, "({left})")?;
        } else {
            write!(f, "{left}")?;
        }

        write!(f, " {op} ")?;

        if matches!(right.expr, Expr::BinOp(_)) {
            write!(f, "({right})")
        } else {
            write!(f, "{right}")
        }
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
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

#[derive(Clone, Debug, Display, PartialEq, From)]
pub enum Expr {
    Lit(Lit),
    TypeCons(TypeConsExpr),
    Var(VarExpr),
    Postfix(PostfixExpr),
    UnOp(UnOpExpr),
    BinOp(BinOpExpr),
    FnCall(FnCallExpr),
}

#[derive(Clone, Debug, Display, PartialEq)]
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
        ExprNode {
            data_type: expr.postfix.type_eval(&expr.inner.data_type),
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
