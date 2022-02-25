mod ext;

use ast::types::DataType;
use serde::{Deserialize, Serialize};

pub use ext::DataTypeExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderMetadata {
    pub resources: Vec<Resource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceKind {
    StorageBuffer,
    UniformBuffer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub kind: ResourceKind,
    pub group: u32,
    pub binding: u32,
    pub description: DataType,
    pub init: Option<Vec<u8>>,
}
