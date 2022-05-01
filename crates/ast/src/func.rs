use std::fmt::Display;

use crate::stmt::Statement;
use crate::types::DataType;

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
