use std::fmt::{Display, Write};

use indenter::indented;

use crate::types::{DataType, ScalarType};
use crate::{ExprNode, Postfix};

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentLhs {
    Underscore,
    Simple(String, Vec<Postfix>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentOp {
    Simple,
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    And,
    Or,
    Xor,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Else {
    If(ExprNode, Vec<Statement>, Option<Box<Else>>),
    Else(Vec<Statement>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForInit {
    pub name: String,
    pub value: Option<ExprNode>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForUpdate {
    Increment(String),
    Decrement(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    LetDecl(String, ExprNode),
    VarDecl(String, ExprNode),
    Assignment(AssignmentLhs, AssignmentOp, ExprNode),
    Compound(Vec<Statement>),
    If(ExprNode, Vec<Statement>, Option<Box<Else>>),
    Return(Option<ExprNode>),
    Loop(Vec<Statement>),
    Break,
    Switch(ExprNode, Vec<(ExprNode, Vec<Statement>)>, Vec<Statement>),
    ForLoop(
        Option<ForInit>,
        Option<ExprNode>,
        Option<ForUpdate>,
        Vec<Statement>,
    ),
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

impl Display for Else {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "else ")?;

        match self {
            Else::If(cond, stmts, els) => {
                writeln!(f, "if ({cond}) {{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")?;

                if let Some(els) = els {
                    write!(f, " {els}")?;
                }
            }
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

impl Display for AssignmentOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentOp::Simple => write!(f, "="),
            AssignmentOp::Plus => write!(f, "+="),
            AssignmentOp::Minus => write!(f, "-="),
            AssignmentOp::Times => write!(f, "*="),
            AssignmentOp::Divide => write!(f, "/="),
            AssignmentOp::Mod => write!(f, "%="),
            AssignmentOp::And => write!(f, "&="),
            AssignmentOp::Or => write!(f, "|="),
            AssignmentOp::Xor => write!(f, "^="),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::LetDecl(name, value) => write!(f, "let {name} = {value};"),
            Statement::VarDecl(name, value) => write!(f, "var {name} = {value};"),
            Statement::Assignment(lhs, op, rhs) => write!(f, "{lhs} {op} {rhs};"),
            Statement::Compound(stmts) => {
                writeln!(f, "{{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
            Statement::If(cond, stmts, els) => {
                writeln!(f, "if ({}) {{", cond)?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")?;

                if let Some(els) = els {
                    write!(f, " {els}")?;
                }

                Ok(())
            }
            Statement::Return(value) => {
                write!(f, "return")?;

                if let Some(value) = value {
                    write!(f, " {}", value)?;
                }

                write!(f, ";")
            }
            Statement::Loop(stmts) => {
                writeln!(f, "loop {{")?;

                for stmt in stmts {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
            Statement::Break => write!(f, "break;"),
            Statement::Switch(selector, cases, default) => {
                writeln!(f, "switch ({selector}) {{")?;

                for (expr, block) in cases {
                    writeln!(indented(f), "case {expr}: {{")?;

                    for stmt in block {
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
            Statement::ForLoop(init, cond, update, body) => {
                write!(f, "for (")?;

                if let Some(init) = init {
                    write!(
                        f,
                        "var {}: {}",
                        init.name,
                        DataType::Scalar(ScalarType::I32)
                    )?;
                    if let Some(value) = &init.value {
                        write!(f, " = {value}")?;
                    }
                }

                write!(f, "; ")?;

                if let Some(cond) = cond {
                    write!(f, "{cond}")?;
                }

                write!(f, "; ")?;

                if let Some(update) = update {
                    match update {
                        ForUpdate::Increment(name) => write!(f, "{name} += 1")?,
                        ForUpdate::Decrement(name) => write!(f, "{name} -= 1")?,
                    }
                }

                writeln!(f, ") {{")?;

                for stmt in body {
                    writeln!(indented(f), "{}", stmt)?;
                }

                write!(f, "}}")
            }
        }
    }
}
