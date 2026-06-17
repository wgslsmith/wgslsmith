use std::fmt::Display;

use derive_more::Display;

use crate::builtins::BuiltinValue;
use crate::stmt::Statement;
use crate::types::DataType;
use crate::{InterpolationSampling, InterpolationType};

#[derive(Debug, Display, PartialEq, Eq)]
pub enum ShaderStage {
    #[display("compute")]
    Compute,
    #[display("vertex")]
    Vertex,
    #[display("fragment")]
    Fragment,
}

#[derive(Debug, Display, PartialEq)]
pub enum FnAttr {
    #[display("stage({_0})")]
    Stage(ShaderStage),
    #[display("workgroup_size({})", crate::FmtArgs(_0.as_slice()))]
    WorkgroupSize(Vec<crate::ExprNode>),
    #[display("must_use")]
    MustUse,
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnIOAttr {
    #[display("builtin({_0})")]
    Builtin(BuiltinValue),
    #[display("invariant")]
    Invariant,
    #[display("location({_0})")]
    Location(u32),
    #[display("interpolate({_0}{})", _1.as_ref().map(|s| format!(", {s}")).unwrap_or_default())]
    Interpolate(InterpolationType, Option<InterpolationSampling>),
}

#[derive(Debug, Display, PartialEq, Eq)]
#[display("{}{name}: {data_type}", InlineAttrs(attrs))]
pub struct FnInput {
    pub attrs: Vec<FnIOAttr>,
    pub name: String,
    pub data_type: DataType,
}

impl FnInput {
    pub fn new(name: impl Into<String>, data_type: impl Into<DataType>) -> Self {
        Self {
            attrs: vec![],
            name: name.into(),
            data_type: data_type.into(),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq)]
#[display("{}{data_type}", InlineAttrs(attrs))]
pub struct FnOutput {
    pub attrs: Vec<FnIOAttr>,
    pub data_type: DataType,
}

impl FnOutput {
    pub fn new(data_type: impl Into<DataType>) -> Self {
        Self {
            attrs: vec![],
            data_type: data_type.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FnDecl {
    pub attrs: Vec<FnAttr>,
    pub name: String,
    pub inputs: Vec<FnInput>,
    pub output: Option<FnOutput>,
    pub body: Vec<Statement>,
}

struct InlineAttrs<'a, T>(&'a [T]);

impl<T: Display> Display for InlineAttrs<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for attr in self.0 {
            write!(f, "@{attr} ")?;
        }

        Ok(())
    }
}
