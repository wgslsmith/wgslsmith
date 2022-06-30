use bincode::{Decode, Encode};

#[derive(Debug, Decode, Encode)]
pub struct Request {
    pub shader: String,
    pub metadata: String,
    pub configs: Vec<String>,
}

#[derive(Debug, Decode, Encode)]
pub struct Response {
    pub exit_code: i32,
    pub output: String,
}
