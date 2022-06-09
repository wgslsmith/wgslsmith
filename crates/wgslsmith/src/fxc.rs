use std::net::TcpStream;

#[derive(Debug, bincode::Encode)]
struct Request {
    hlsl: String,
}

#[derive(Debug, bincode::Decode)]
pub enum Response {
    Success,
    Failure(String),
}

pub fn validate_hlsl(server: &str, hlsl: String) -> eyre::Result<Response> {
    let mut stream = TcpStream::connect(server)?;
    bincode::encode_into_std_write(Request { hlsl }, &mut stream, bincode::config::standard())?;
    bincode::decode_from_std_read(&mut stream, bincode::config::standard()).map_err(Into::into)
}
