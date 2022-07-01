use bincode::{Decode, Encode};
use types::{Config, ConfigId};

#[derive(Debug, Decode, Encode)]
pub enum Request {
    List,
    Run {
        shader: String,
        metadata: String,
        configs: Vec<ConfigId>,
    },
}

#[derive(Debug, Decode, Encode)]
pub struct ListResponse {
    pub configs: Vec<Config>,
}

#[derive(Debug, Decode, Encode)]
pub struct RunResponse {
    pub exit_code: i32,
    pub output: String,
}
