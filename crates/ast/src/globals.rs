use std::fmt::Display;

use crate::types::DataType;
use crate::{AttrList, ExprNode};

#[derive(Debug, PartialEq, Eq)]
pub enum GlobalVarAttr {
    Binding(i32),
    Group(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum StorageClass {
    Function,
    Private,
    WorkGroup,
    Uniform,
    Storage,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VarQualifier {
    pub storage_class: StorageClass,
    pub access_mode: Option<AccessMode>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobalVarDecl {
    pub attrs: AttrList<GlobalVarAttr>,
    pub qualifier: Option<VarQualifier>,
    pub name: String,
    pub data_type: DataType,
    pub initializer: Option<ExprNode>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobalConstDecl {
    pub name: String,
    pub data_type: DataType,
    pub initializer: ExprNode,
}

impl Display for GlobalVarAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalVarAttr::Binding(v) => write!(f, "binding({})", v),
            GlobalVarAttr::Group(v) => write!(f, "group({})", v),
        }
    }
}

impl Display for StorageClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StorageClass::Function => "function",
            StorageClass::Private => "private",
            StorageClass::WorkGroup => "workgroup",
            StorageClass::Uniform => "uniform",
            StorageClass::Storage => "storage",
        })
    }
}

impl Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AccessMode::Read => "read",
            AccessMode::Write => "write",
            AccessMode::ReadWrite => "read_write",
        })
    }
}

impl Display for GlobalVarDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attrs)?;
        write!(f, "var")?;

        if let Some(qualifier) = &self.qualifier {
            write!(f, "<{}", qualifier.storage_class)?;
            if let Some(access_mode) = &qualifier.access_mode {
                write!(f, ", {}", access_mode)?;
            }
            write!(f, ">")?;
        }

        write!(f, " {}: {}", self.name, self.data_type)?;

        if let Some(initializer) = &self.initializer {
            write!(f, " = {}", initializer)?;
        }

        writeln!(f, ";")
    }
}

impl Display for GlobalConstDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "const {}: {} = {};",
            self.name, self.data_type, self.initializer
        )
    }
}
