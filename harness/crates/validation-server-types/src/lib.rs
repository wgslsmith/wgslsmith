use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum Backend {
    Hlsl,
    Msl,
}

#[derive(Debug, Encode, Decode)]
pub enum Request {
    GetCount,
    ResetCount,
    Validate { backend: Backend, source: String },
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
