use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use bincode::Decode;
use harness_server_types::{ListResponse, Request, RunResponse};
use harness_types::ConfigId;

pub fn query_configs(server: &str) -> eyre::Result<ListResponse> {
    req(server, Request::List)
}

pub fn exec_shader(server: &str, shader: String, metadata: String) -> eyre::Result<RunResponse> {
    exec_shader_with(server, shader, metadata, vec![])
}

pub fn exec_shader_with(
    server: &str,
    shader: String,
    metadata: String,
    configs: Vec<ConfigId>,
) -> eyre::Result<RunResponse> {
    req(
        server,
        Request::Run {
            shader,
            metadata,
            configs,
        },
    )
}

fn req<T: Decode>(server: &str, req: Request) -> eyre::Result<T> {
    let address = SocketAddr::from_str(server)?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))?;
    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard())?;
    bincode::decode_from_std_read(&mut stream, bincode::config::standard()).map_err(Into::into)
}
