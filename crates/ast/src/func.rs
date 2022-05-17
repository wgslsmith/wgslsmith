use std::fmt::Display;

use derive_more::Display;

use crate::stmt::Statement;
use crate::types::DataType;

#[derive(Debug, Display, PartialEq, Eq)]
pub enum ShaderStage {
    #[display(fmt = "compute")]
    Compute,
    #[display(fmt = "vertex")]
    Vertex,
    #[display(fmt = "fragment")]
    Fragment,
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnAttr {
    #[display(fmt = "stage({_0})")]
    Stage(ShaderStage),
    #[display(fmt = "workgroup_size({_0})")]
    WorkgroupSize(u32),
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnInputAttr {}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum FnOutputAttr {}

#[derive(Debug, Display, PartialEq, Eq)]
#[display(fmt = "{} {name}: {data_type}", "InlineAttrs(attrs)")]
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
#[display(fmt = "{} {data_type}", "InlineAttrs(attrs)")]
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

impl<'a, T: Display> Display for InlineAttrs<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut attrs = self.0.iter();

        if let Some(attr) = attrs.next() {
            write!(f, "@{attr}")?;
        }

        for attr in attrs {
            write!(f, " @{attr}")?;
        }

        Ok(())
    }
}
