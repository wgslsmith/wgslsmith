use std::time::Duration;

use bincode::{Decode, Encode};
use reflection_types::PipelineDescription;
use types::{Config, ConfigId};

#[derive(Debug, Decode, Encode)]
pub enum Request {
    List,
    Run(RunRequest),
}

#[derive(Debug, Decode, Encode)]
pub struct ListResponse {
    pub configs: Vec<Config>,
}

#[derive(Debug, Decode, Encode)]
pub struct RunRequest {
    pub shader: String,
    pub workgroups: u32,
    pub flow: bool,
    pub pipeline_desc: PipelineDescription,
    pub configs: Vec<ConfigId>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Decode, Encode)]
pub enum RunMessage {
    UsingDefaultConfigs(Vec<ConfigId>),
    ExecStart(ConfigId),
    ExecSuccess(Vec<Vec<u8>>, Option<Vec<u32>>),
    ExecFailure(Vec<u8>),
    ExecTimeout,
    End(Result<(), RunError>),
}

#[derive(Debug, Decode, Encode)]
pub enum RunError {
    NoDefaultConfigs,
    InternalServerError,
}
