use std::net::TcpStream;

use bincode::Decode;

pub use validation_server_types::*;

pub fn get_count(server: &str) -> eyre::Result<u64> {
    let mut stream = TcpStream::connect(server)?;
    req(&mut stream, Request::GetCount)
}

pub fn reset_count(server: &str) -> eyre::Result<()> {
    let mut stream = TcpStream::connect(server)?;
    bincode::encode_into_std_write(
        Request::ResetCount,
        &mut stream,
        bincode::config::standard(),
    )?;
    Ok(())
}

pub fn validate(server: &str, backend: Backend, source: String) -> eyre::Result<ValidateResponse> {
    let mut stream = TcpStream::connect(server)?;
    req(&mut stream, Request::Validate { backend, source })
}

fn req<T: Decode>(stream: &mut TcpStream, req: Request) -> eyre::Result<T> {
    bincode::encode_into_std_write(req, stream, bincode::config::standard())?;
    bincode::decode_from_std_read(stream, bincode::config::standard()).map_err(Into::into)
}
