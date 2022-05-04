use serde::{Deserialize, Serialize};

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
    pub size: usize,
    pub init: Option<Vec<u8>>,
}
