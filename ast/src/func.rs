use std::fmt::{Display, Write};

use indenter::indented;

use crate::stmt::Statement;
use crate::types::DataType;
use crate::AttrList;

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
    pub attrs: AttrList<FnInputAttr>,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FnOutput {
    pub attrs: AttrList<FnOutputAttr>,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FnDecl {
    pub attrs: AttrList<FnAttr>,
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

impl Display for FnDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attrs)?;
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
