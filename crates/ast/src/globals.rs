use derive_more::Display;

use crate::types::DataType;
use crate::ExprNode;

#[derive(Debug, Display, PartialEq, Eq)]
pub enum GlobalVarAttr {
    #[display(fmt = "binding({_0})")]
    Binding(i32),
    #[display(fmt = "group({_0})")]
    Group(i32),
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum StorageClass {
    #[display(fmt = "function")]
    Function,
    #[display(fmt = "private")]
    Private,
    #[display(fmt = "workgroup")]
    WorkGroup,
    #[display(fmt = "uniform")]
    Uniform,
    #[display(fmt = "storage")]
    Storage,
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum AccessMode {
    #[display(fmt = "read")]
    Read,
    #[display(fmt = "write")]
    Write,
    #[display(fmt = "read_write")]
    ReadWrite,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VarQualifier {
    pub storage_class: StorageClass,
    pub access_mode: Option<AccessMode>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobalVarDecl {
    pub attrs: Vec<GlobalVarAttr>,
    pub qualifier: Option<VarQualifier>,
    pub name: String,
    pub data_type: DataType,
    pub initializer: Option<ExprNode>,
}

impl GlobalVarDecl {
    pub fn group_index(&self) -> Option<u32> {
        self.attrs.iter().find_map(|it| {
            if let GlobalVarAttr::Group(v) = it {
                Some(*v as u32)
            } else {
                None
            }
        })
    }

    pub fn binding_index(&self) -> Option<u32> {
        self.attrs.iter().find_map(|it| {
            if let GlobalVarAttr::Binding(v) = it {
                Some(*v as u32)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobalConstDecl {
    pub name: String,
    pub data_type: DataType,
    pub initializer: ExprNode,
}
