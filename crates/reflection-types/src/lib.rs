use bincode::{Decode, Encode};

pub struct ResourceData<'a> {
    pub name: &'a str,
    pub group: u32,
    pub binding: u32,
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct PipelineDescription {
    pub resources: Vec<PipelineResource>,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub enum ResourceKind {
    StorageBuffer,
    UniformBuffer,
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct PipelineResource {
    pub name: String,
    pub kind: ResourceKind,
    pub group: u32,
    pub binding: u32,
    pub init: Option<Vec<u8>>,
    pub size: u32,
}
