use std::fmt::{Display, Write};

use derive_more::{Display, From};
use indenter::indented;

use crate::types::DataType;
use crate::{ExprNode, Postfix};

#[derive(Debug, Display, PartialEq, Eq)]
#[display(fmt = "let {ident} = {initializer}")]
pub struct LetDeclStatement {
    pub ident: String,
    pub initializer: ExprNode,
}

impl LetDeclStatement {
    pub fn new(ident: String, initializer: ExprNode) -> Self {
        Self { ident, initializer }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VarDeclStatement {
    pub ident: String,
    pub data_type: Option<DataType>,
    pub initializer: Option<ExprNode>,
}

impl VarDeclStatement {
    pub fn new(ident: String, data_type: Option<DataType>, initializer: Option<ExprNode>) -> Self {
        Self {
            ident,
            data_type,
            initializer,
        }
    }
}

impl Display for VarDeclStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let VarDeclStatement {
            ident,
            data_type,
            initializer,
        } = self;

        write!(f, "var {ident}")?;

        if let Some(data_type) = data_type {
            write!(f, ": {data_type}")?;
        }

        if let Some(initializer) = initializer {
            write!(f, " = {initializer}")?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentLhs {
    Underscore,
    Simple(String, Vec<Postfix>),
}

impl Display for AssignmentLhs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentLhs::Underscore => f.write_char('_'),
            AssignmentLhs::Simple(name, postfixes) => {
                f.write_str(name)?;

                for postfix in postfixes {
                    match postfix {
                        Postfix::ArrayIndex(index) => write!(f, "[{}]", index)?,
                        Postfix::Member(field) => write!(f, ".{}", field)?,
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum AssignmentOp {
    #[display(fmt = "=")]
    Simple,
    #[display(fmt = "+=")]
    Plus,
    #[display(fmt = "-=")]
    Minus,
    #[display(fmt = "*=")]
    Times,
    #[display(fmt = "/=")]
    Divide,
    #[display(fmt = "%=")]
    Mod,
    #[display(fmt = "&=")]
    And,
    #[display(fmt = "|=")]
    Or,
    #[display(fmt = "^=")]
    Xor,
}

#[derive(Debug, Display, PartialEq, Eq)]
#[display(fmt = "{lhs} {op} {rhs}")]
pub struct AssignmentStatement {
    pub lhs: AssignmentLhs,
    pub op: AssignmentOp,
    pub rhs: ExprNode,
}

impl AssignmentStatement {
    pub fn new(lhs: AssignmentLhs, op: AssignmentOp, rhs: ExprNode) -> Self {
        Self { lhs, op, rhs }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Else {
    If(IfStatement),
    Else(Vec<Statement>),
}

impl Display for Else {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "else ")?;

        match self {
            Else::If(stmt) => stmt.fmt(f)?,
            Else::Else(stmts) => {
                writeln!(f, "{{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfStatement {
    pub condition: ExprNode,
    pub body: Vec<Statement>,
    pub else_: Option<Box<Else>>,
}

impl IfStatement {
    pub fn new(condition: ExprNode, body: Vec<Statement>) -> Self {
        Self {
            condition,
            body,
            else_: None,
        }
    }

    pub fn with_else(mut self, else_: impl Into<Option<Else>>) -> Self {
        self.else_ = else_.into().map(Box::new);
        self
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let IfStatement {
            condition,
            body,
            else_,
        } = self;

        writeln!(f, "if ({condition}) {{")?;

        for stmt in body {
            writeln!(indented(f), "{}", stmt)?;
        }

        write!(f, "}}")?;

        if let Some(else_) = else_ {
            write!(f, " {else_}")?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReturnStatement {
    pub value: Option<ExprNode>,
}

impl ReturnStatement {
    pub fn new(value: Option<ExprNode>) -> Self {
        Self { value }
    }
}

impl Display for ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return")?;

        if let Some(value) = &self.value {
            write!(f, " {}", value)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoopStatement {
    pub body: Vec<Statement>,
}

impl LoopStatement {
    pub fn new(body: Vec<Statement>) -> Self {
        Self { body }
    }
}

impl Display for LoopStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "loop {{")?;

        for stmt in &self.body {
            writeln!(indented(f), "{}", stmt)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SwitchStatement {
    pub selector: ExprNode,
    pub cases: Vec<SwitchCase>,
    pub default: Vec<Statement>,
}

impl SwitchStatement {
    pub fn new(selector: ExprNode, cases: Vec<SwitchCase>, default: Vec<Statement>) -> Self {
        Self {
            selector,
            cases,
            default,
        }
    }
}

impl Display for SwitchStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SwitchStatement {
            selector,
            cases,
            default,
        } = self;

        writeln!(f, "switch ({selector}) {{")?;

        for SwitchCase { selector, body } in cases {
            writeln!(indented(f), "case {selector}: {{")?;

            for stmt in body {
                writeln!(indented(&mut indented(f)), "{}", stmt)?;
            }

            writeln!(indented(f), "}}")?;
        }

        writeln!(indented(f), "default: {{")?;

        for stmt in default {
            writeln!(indented(&mut indented(f)), "{}", stmt)?;
        }

        writeln!(indented(f), "}}")?;

        write!(f, "}}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SwitchCase {
    pub selector: ExprNode,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForLoopInit {
    VarDecl(VarDeclStatement),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForLoopUpdate {
    Assignment(AssignmentStatement),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForLoopHeader {
    pub init: Option<ForLoopInit>,
    pub condition: Option<ExprNode>,
    pub update: Option<ForLoopUpdate>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForLoopStatement {
    pub header: Box<ForLoopHeader>,
    pub body: Vec<Statement>,
}

impl ForLoopStatement {
    pub fn new(header: ForLoopHeader, body: Vec<Statement>) -> Self {
        Self {
            header: Box::new(header),
            body,
        }
    }
}

impl Display for ForLoopStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ForLoopStatement { header, body } = self;

        write!(f, "for (")?;

        if let Some(init) = &header.init {
            match init {
                ForLoopInit::VarDecl(stmt) => stmt.fmt(f)?,
            }
        }

        write!(f, "; ")?;

        if let Some(condition) = &header.condition {
            write!(f, "{condition}")?;
        }

        write!(f, "; ")?;

        if let Some(update) = &header.update {
            match update {
                ForLoopUpdate::Assignment(stmt) => stmt.fmt(f)?,
            }
        }

        writeln!(f, ") {{")?;

        for stmt in body {
            writeln!(indented(f), "{}", stmt)?;
        }

        write!(f, "}}")
    }
}

#[derive(Debug, PartialEq, Eq, From)]
pub enum Statement {
    LetDecl(LetDeclStatement),
    VarDecl(VarDeclStatement),
    Assignment(AssignmentStatement),
    Compound(Vec<Statement>),
    If(IfStatement),
    Return(ReturnStatement),
    Loop(LoopStatement),
    Break,
    Switch(SwitchStatement),
    ForLoop(ForLoopStatement),
}

impl Statement {
    /// Extracts the inner statements from a `Statement::CompoundStatement`.
    ///
    /// This will panic if `self` is not a `Statement::CompoundStatement`.
    pub fn into_compount_statement(self) -> Vec<Statement> {
        match self {
            Statement::Compound(stmts) => stmts,
            _ => unreachable!(),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::LetDecl(stmt) => write!(f, "{stmt};"),
            Statement::VarDecl(stmt) => write!(f, "{stmt};"),
            Statement::Assignment(stmt) => write!(f, "{stmt};"),
            Statement::Compound(stmts) => {
                writeln!(f, "{{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
            Statement::If(stmt) => stmt.fmt(f),
            Statement::Return(stmt) => write!(f, "{stmt};"),
            Statement::Loop(stmt) => stmt.fmt(f),
            Statement::Break => write!(f, "break;"),
            Statement::Switch(stmt) => stmt.fmt(f),
            Statement::ForLoop(stmt) => stmt.fmt(f),
        }
    }
}
