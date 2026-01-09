use std::fmt::Display;

use derive_more::Display;

use crate::stmt::Statement;
use crate::types::DataType;

#[derive(Debug, Display, PartialEq, Eq)]
pub enum ShaderStage {
    #[display("compute")]
    Compute,
    #[display("vertex")]
    Vertex,
    #[display("fragment")]
    Fragment,
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnAttr {
    #[display("stage({_0})")]
    Stage(ShaderStage),
    #[display("workgroup_size({_0})")]
    WorkgroupSize(u32),
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnInputAttr {}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnOutputAttr {}

#[derive(Debug, Display, PartialEq, Eq)]
#[display("{}{name}: {data_type}", InlineAttrs(attrs))]
pub struct FnInput {
    pub attrs: Vec<FnInputAttr>,
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
    pub attrs: Vec<FnOutputAttr>,
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
