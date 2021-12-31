pub mod types;

use std::fmt::{Display, Write};

use indenter::indented;
use types::{DataType, ScalarType};

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
        *t
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
            | BinOp::RShift => *l,

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
pub enum Expr {
    Lit(Lit),
    TypeCons(DataType, Vec<ExprNode>),
    Var(String),
    UnOp(UnOp, Box<ExprNode>),
    BinOp(BinOp, Box<ExprNode>, Box<ExprNode>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExprNode {
    pub data_type: DataType,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentLhs {
    Underscore,
    SimpleVar(String),
    ArrayIndex { name: String, index: ExprNode },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    LetDecl(String, ExprNode),
    VarDecl(String, ExprNode),
    Assignment(AssignmentLhs, ExprNode),
    Compound(Vec<Statement>),
    If(ExprNode, Vec<Statement>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Compute,
    Vertex,
    Fragment,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FnAttr {
    Stage(ShaderStage),
    WorkgroupSize(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FnInputAttr {}

#[derive(Debug, PartialEq, Eq)]
pub enum FnOutputAttr {}

#[derive(Debug, PartialEq, Eq)]
pub struct FnInput {
    pub attrs: Vec<FnInputAttr>,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FnOutput {
    pub attrs: Vec<FnOutputAttr>,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FnDecl {
    pub attrs: Vec<FnAttr>,
    pub name: String,
    pub inputs: Vec<FnInput>,
    pub output: Option<FnOutput>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub entrypoint: FnDecl,
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
                t.fmt(f)?;
                f.write_char('(')?;

                for (i, e) in args.iter().enumerate() {
                    e.fmt(f)?;
                    if i != args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(')')
            }
            Expr::Var(name) => name.fmt(f),
            Expr::UnOp(op, e) => write!(f, "{}({})", op, e),
            Expr::BinOp(op, l, r) => write!(f, "({}) {} ({})", l, op, r),
        }
    }
}

impl Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expr.fmt(f)
    }
}

impl Display for AssignmentLhs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentLhs::Underscore => f.write_char('_'),
            AssignmentLhs::SimpleVar(n) => f.write_str(n),
            AssignmentLhs::ArrayIndex { name, index } => write!(f, "{}[{}]", name, index),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::LetDecl(name, value) => write!(f, "let {} = {};", name, value),
            Statement::VarDecl(name, value) => write!(f, "var {} = {};", name, value),
            Statement::Assignment(lhs, rhs) => write!(f, "{} = {};", lhs, rhs),
            Statement::Compound(stmts) => {
                writeln!(f, "{{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
            Statement::If(cond, stmts) => {
                writeln!(f, "if ({}) {{", cond)?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
        }
    }
}

impl Display for ShaderStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ShaderStage::Compute => "compute",
            ShaderStage::Vertex => "vertex",
            ShaderStage::Fragment => "fragment",
        })
    }
}

impl Display for FnAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FnAttr::Stage(stage) => write!(f, "stage({})", stage),
            FnAttr::WorkgroupSize(size) => write!(f, "workgroup_size({})", size),
        }
    }
}

impl Display for FnInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Write attributes
        write!(f, "{}: {}", self.name, self.data_type)
    }
}

impl Display for FnOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Write attributes
        self.data_type.fmt(f)
    }
}

impl Display for FnDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", include_str!("prelude.wgsl"))?;

        if !self.attrs.is_empty() {
            f.write_str("[[")?;

            for (i, attr) in self.attrs.iter().enumerate() {
                attr.fmt(f)?;
                if i != self.attrs.len() - 1 {
                    f.write_str(", ")?;
                }
            }

            writeln!(f, "]]")?;
        }

        write!(f, "fn {}(", self.name)?;

        for (i, param) in self.inputs.iter().enumerate() {
            param.fmt(f)?;
            if i != self.inputs.len() - 1 {
                f.write_str(", ")?;
            }
        }

        f.write_str(") ")?;

        if let Some(output) = &self.output {
            f.write_str("-> ")?;
            output.fmt(f)?;
            f.write_char(' ')?;
        }

        writeln!(f, "{{")?;

        for stmt in &self.body {
            writeln!(indented(f), "{}", stmt)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.entrypoint.fmt(f)
    }
}
