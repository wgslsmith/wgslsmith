use std::net::TcpStream;

use bincode::Decode;

pub use validation_server_types::*;

pub fn validate(server: &str, backend: Backend, source: String) -> eyre::Result<ValidateResponse> {
    let mut stream = TcpStream::connect(server)?;
    req(&mut stream, Request::Validate { backend, source })
}

fn req<T: Decode>(stream: &mut TcpStream, req: Request) -> eyre::Result<T> {
    bincode::encode_into_std_write(req, stream, bincode::config::standard())?;
    bincode::decode_from_std_read(stream, bincode::config::standard()).map_err(Into::into)
}
