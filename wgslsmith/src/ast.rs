use std::fmt::{Display, Write};

use crate::types::DataType;

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
    Var(String),
    UnOp(UnOp, Box<ExprNode>),
    BinOp(BinOp, Box<ExprNode>, Box<ExprNode>),
}

#[derive(Debug)]
pub struct ExprNode {
    pub data_type: DataType,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum AssignmentLhs {
    Underscore,
    ArrayIndex { name: String, index: ExprNode },
}

#[derive(Debug)]
pub enum Statement {
    VarDecl(String, ExprNode),
    Assignment(AssignmentLhs, ExprNode),
}

#[derive(Debug)]
pub enum ShaderStage {
    Compute,
    Vertex,
    Fragment,
}

#[derive(Debug)]
pub enum FnAttr {
    Stage(ShaderStage),
    WorkgroupSize(u32),
}

#[derive(Debug)]
pub enum FnInputAttr {}

#[derive(Debug)]
pub enum FnOutputAttr {}

#[derive(Debug)]
pub struct FnInput {
    pub attrs: Vec<FnInputAttr>,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug)]
pub struct FnOutput {
    pub attrs: Vec<FnOutputAttr>,
    pub data_type: DataType,
}

#[derive(Debug)]
pub struct FnDecl {
    pub attrs: Vec<FnAttr>,
    pub name: String,
    pub inputs: Vec<FnInput>,
    pub output: Option<FnOutput>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
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
            Expr::Var(name) => name.fmt(f),
            Expr::UnOp(op, e) => write!(f, "{}({})", op, e),
            Expr::BinOp(op, l, r) => {
                if let BinOp::Divide = op {
                    write!(f, "safe_divide_{}({}, {})", l.data_type, l, r)
                } else if let BinOp::Mod = op {
                    write!(f, "safe_mod_{}({}, {})", l.data_type, l, r)
                } else {
                    write!(f, "({}){}({})", l, op, r)
                }
            }
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
            AssignmentLhs::ArrayIndex { name, index } => write!(f, "{}[{}]", name, index),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::VarDecl(name, value) => write!(f, "let {} = {}", name, value),
            Statement::Assignment(lhs, rhs) => write!(f, "{} = {}", lhs, rhs),
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
            // TODO: Do indentation properly
            writeln!(f, "    {};", stmt)?;
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
