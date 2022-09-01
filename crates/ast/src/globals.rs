use derive_more::Display;

use crate::types::DataType;
use crate::ExprNode;

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum GlobalVarAttr {
    #[display(fmt = "binding({_0})")]
    Binding(i32),
    #[display(fmt = "group({_0})")]
    Group(i32),
}

#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Eq)]
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

impl StorageClass {
    pub fn default_access_mode(&self) -> AccessMode {
        match self {
            StorageClass::Function => AccessMode::ReadWrite,
            StorageClass::Private => AccessMode::ReadWrite,
            StorageClass::WorkGroup => AccessMode::ReadWrite,
            StorageClass::Uniform => AccessMode::Read,
            StorageClass::Storage => AccessMode::Read,
        }
    }
}

#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Eq)]
pub enum AccessMode {
    #[display(fmt = "read")]
    Read,
    #[display(fmt = "write")]
    Write,
    #[display(fmt = "read_write")]
    ReadWrite,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VarQualifier {
    pub storage_class: StorageClass,
    pub access_mode: Option<AccessMode>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct GlobalConstDecl {
    pub name: String,
    pub data_type: DataType,
    pub initializer: ExprNode,
}
