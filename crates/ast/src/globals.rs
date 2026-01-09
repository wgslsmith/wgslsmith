use derive_more::Display;

use crate::types::DataType;
use crate::ExprNode;

#[derive(Debug, Display, PartialEq, Eq)]
pub enum GlobalVarAttr {
    #[display("binding({_0})")]
    Binding(i32),
    #[display("group({_0})")]
    Group(i32),
}

#[derive(Clone, Copy, Debug, Display, Hash, PartialEq, Eq)]
pub enum StorageClass {
    #[display("function")]
    Function,
    #[display("private")]
    Private,
    #[display("workgroup")]
    WorkGroup,
    #[display("uniform")]
    Uniform,
    #[display("storage")]
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
    #[display("read")]
    Read,
    #[display("write")]
    Write,
    #[display("read_write")]
    ReadWrite,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VarQualifier {
    pub storage_class: StorageClass,
    pub access_mode: Option<AccessMode>,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct GlobalConstDecl {
    pub name: String,
    pub data_type: DataType,
    pub initializer: ExprNode,
}
