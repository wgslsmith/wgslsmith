use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum Request {
    GetCount,
    Validate { hlsl: String },
}

#[derive(Debug, Encode, Decode)]
pub struct GetCountResponse {
    pub count: u64,
}

#[derive(Debug, Encode, Decode)]
pub enum ValidateResponse {
    Success,
    Failure(String),
}
