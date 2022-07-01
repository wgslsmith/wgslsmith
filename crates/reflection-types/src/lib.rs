pub struct ResourceData<'a> {
    pub name: &'a str,
    pub group: u32,
    pub binding: u32,
}

#[derive(Debug)]
pub struct PipelineDescription {
    pub resources: Vec<PipelineResource>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceKind {
    StorageBuffer,
    UniformBuffer,
}

#[derive(Debug)]
pub struct PipelineResource {
    pub name: String,
    pub kind: ResourceKind,
    pub group: u32,
    pub binding: u32,
    pub init: Option<Vec<u8>>,
    pub type_desc: common::Type,
}
