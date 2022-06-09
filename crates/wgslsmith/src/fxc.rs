use std::net::TcpStream;

use bincode::Decode;
pub use fxc_server_types::{Request, ValidateResponse};

pub fn get_count(server: &str) -> eyre::Result<u64> {
    let mut stream = TcpStream::connect(server)?;
    req(&mut stream, Request::GetCount)
}

pub fn validate_hlsl(server: &str, hlsl: String) -> eyre::Result<ValidateResponse> {
    let mut stream = TcpStream::connect(server)?;
    req(&mut stream, Request::Validate { hlsl })
}

fn req<T: Decode>(stream: &mut TcpStream, req: Request) -> eyre::Result<T> {
    bincode::encode_into_std_write(req, stream, bincode::config::standard())?;
    bincode::decode_from_std_read(stream, bincode::config::standard()).map_err(Into::into)
}
