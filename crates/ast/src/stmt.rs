use std::fmt::{Display, Write};

use derive_more::{Display, From};
use indenter::indented;

use crate::types::DataType;
use crate::{ExprNode, Postfix};

#[derive(Debug, Display, PartialEq)]
#[display("let {ident} = {initializer}")]
pub struct LetDeclStatement {
    pub ident: String,
    pub initializer: ExprNode,
}

impl LetDeclStatement {
    pub fn new(ident: impl Into<String>, initializer: impl Into<ExprNode>) -> Self {
        Self {
            ident: ident.into(),
            initializer: initializer.into(),
        }
    }

    pub fn inferred_type(&self) -> &DataType {
        // If the type of the initializer expression is a reference, then we infer the declaration
        // type to be the target type of the reference. Otherwise it is simply the type of the initializer.
        if let DataType::Ref(view) = &self.initializer.data_type {
            view.inner.as_ref()
        } else {
            &self.initializer.data_type
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VarDeclStatement {
    pub ident: String,
    pub data_type: Option<DataType>,
    pub initializer: Option<ExprNode>,
}

impl VarDeclStatement {
    pub fn new(
        ident: impl Into<String>,
        data_type: Option<DataType>,
        initializer: Option<ExprNode>,
    ) -> Self {
        Self {
            ident: ident.into(),
            data_type,
            initializer,
        }
    }

    pub fn inferred_type(&self) -> &DataType {
        self.data_type.as_ref().unwrap_or_else(|| {
            let initializer = self.initializer.as_ref().unwrap();
            if let DataType::Ref(view) = &initializer.data_type {
                view.inner.as_ref()
            } else {
                &initializer.data_type
            }
        })
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

#[derive(Debug, Display, PartialEq)]
pub enum AssignmentLhs {
    #[display("_")]
    Phony,
    Expr(LhsExprNode),
}

impl AssignmentLhs {
    pub fn name(name: impl Into<String>, data_type: impl Into<DataType>) -> AssignmentLhs {
        LhsExprNode::name(name.into(), data_type.into()).into()
    }

    pub fn array_index(
        name: impl Into<String>,
        array_type: DataType,
        index: ExprNode,
    ) -> AssignmentLhs {
        LhsExprNode::array_index(name.into(), array_type, index).into()
    }

    pub fn member(name: String, data_type: DataType, member: String) -> AssignmentLhs {
        LhsExprNode::member(name, data_type, member).into()
    }
}

#[derive(Debug, Display, PartialEq)]
pub enum LhsExpr {
    Ident(String),
    #[display("({_0}){_1}")]
    Postfix(Box<LhsExprNode>, Postfix),
    #[display("*(_0)")]
    Deref(Box<LhsExprNode>),
    #[display("&(_0)")]
    AddressOf(Box<LhsExprNode>),
}

impl From<LhsExprNode> for AssignmentLhs {
    fn from(node: LhsExprNode) -> Self {
        AssignmentLhs::Expr(node)
    }
}

#[derive(Debug, Display, PartialEq)]
#[display("{expr}")]
pub struct LhsExprNode {
    pub data_type: DataType,
    pub expr: LhsExpr,
}

impl LhsExprNode {
    pub fn name(name: String, data_type: DataType) -> LhsExprNode {
        LhsExprNode {
            data_type,
            expr: LhsExpr::Ident(name),
        }
    }

    pub fn array_index(name: String, array_type: DataType, index: ExprNode) -> LhsExprNode {
        let mem_view = array_type
            .as_memory_view()
            .expect("lhs expression must be a reference type");

        let element_type = if let DataType::Array(ty, _) = mem_view.inner.as_ref() {
            DataType::Ref(mem_view.clone_with_type(ty.as_ref().clone()))
        } else {
            panic!("expected array, got `{}`", mem_view.inner)
        };

        LhsExprNode {
            data_type: element_type,
            expr: LhsExpr::Postfix(
                Box::new(LhsExprNode {
                    data_type: array_type,
                    expr: LhsExpr::Ident(name),
                }),
                Postfix::Index(Box::new(index)),
            ),
        }
    }

    pub fn member(name: String, data_type: DataType, member: String) -> LhsExprNode {
        let member_type = match &data_type {
            DataType::Vector(_, ty) => DataType::Scalar(*ty),
            DataType::Struct(decl) => decl.member_type(&member).cloned().unwrap(),
            _ => panic!("must be array or vector type"),
        };

        LhsExprNode {
            data_type: member_type,
            expr: LhsExpr::Postfix(
                Box::new(LhsExprNode {
                    data_type,
                    expr: LhsExpr::Ident(name),
                }),
                Postfix::Member(member),
            ),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum AssignmentOp {
    #[display("=")]
    Simple,
    #[display("+=")]
    Plus,
    #[display("-=")]
    Minus,
    #[display("*=")]
    Times,
    #[display("/=")]
    Divide,
    #[display("%=")]
    Mod,
    #[display("&=")]
    And,
    #[display("|=")]
    Or,
    #[display("^=")]
    Xor,
}

#[derive(Debug, Display, PartialEq)]
#[display("{lhs} {op} {rhs}")]
pub struct AssignmentStatement {
    pub lhs: AssignmentLhs,
    pub op: AssignmentOp,
    pub rhs: ExprNode,
}

impl AssignmentStatement {
    pub fn new(lhs: AssignmentLhs, op: AssignmentOp, rhs: impl Into<ExprNode>) -> Self {
        Self {
            lhs,
            op,
            rhs: rhs.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct IfStatement {
    pub condition: ExprNode,
    pub body: Vec<Statement>,
    pub else_: Option<Box<Else>>,
}

impl IfStatement {
    pub fn new(condition: impl Into<ExprNode>, body: Vec<Statement>) -> Self {
        Self {
            condition: condition.into(),
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

#[derive(Debug, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<ExprNode>,
}

impl ReturnStatement {
    pub fn new(value: impl Into<ExprNode>) -> Self {
        Self {
            value: Some(value.into()),
        }
    }

    pub fn optional(value: Option<impl Into<ExprNode>>) -> Self {
        Self {
            value: value.map(|it| it.into()),
        }
    }

    pub fn none() -> Self {
        Self { value: None }
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct SwitchCase {
    pub selector: ExprNode,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub enum ForLoopInit {
    VarDecl(VarDeclStatement),
}

#[derive(Debug, PartialEq)]
pub enum ForLoopUpdate {
    Assignment(AssignmentStatement),
}

#[derive(Debug, PartialEq)]
pub struct ForLoopHeader {
    pub init: Option<ForLoopInit>,
    pub condition: Option<ExprNode>,
    pub update: Option<ForLoopUpdate>,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct FnCallStatement {
    pub ident: String,
    pub args: Vec<ExprNode>,
}

impl FnCallStatement {
    pub fn new(ident: String, args: Vec<ExprNode>) -> Self {
        Self { ident, args }
    }
}

impl Display for FnCallStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.ident)?;

        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{arg}")?;
            if i != self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}

#[derive(Debug, PartialEq, From)]
pub enum Statement {
    LetDecl(LetDeclStatement),
    VarDecl(VarDeclStatement),
    Assignment(AssignmentStatement),
    Compound(Vec<Statement>),
    If(IfStatement),
    Return(ReturnStatement),
    Loop(LoopStatement),
    Break,
    Continue,
    Switch(SwitchStatement),
    Fallthrough,
    ForLoop(ForLoopStatement),
    FnCall(FnCallStatement),
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
            Statement::Continue => write!(f, "continue;"),
            Statement::Fallthrough => write!(f, "fallthrough;"),
            Statement::Switch(stmt) => stmt.fmt(f),
            Statement::ForLoop(stmt) => stmt.fmt(f),
            Statement::FnCall(stmt) => write!(f, "{stmt};"),
        }
    }
}
