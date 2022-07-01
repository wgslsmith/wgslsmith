use std::io::Read;
use std::net::TcpStream;

use clap::Parser;
use harness_server_types::{ListResponse, Request, RunResponse};
use harness_types::ConfigId;

pub fn query_configs(server: &str) -> eyre::Result<ListResponse> {
    let mut stream = TcpStream::connect(server).unwrap();

    let req = Request::List;

    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard())?;
    bincode::decode_from_std_read(&mut stream, bincode::config::standard()).map_err(Into::into)
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
    let mut stream = TcpStream::connect(server).unwrap();

    let req = Request::Run {
        shader,
        metadata,
        configs,
    };

    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard())?;
    bincode::decode_from_std_read(&mut stream, bincode::config::standard()).map_err(Into::into)
}

#[derive(Parser)]
pub struct Options {
    server: String,
    metadata: String,
}

#[allow(unused)]
pub fn run(options: Options) -> eyre::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let res = exec_shader(&options.server, input, options.metadata)?;
    println!("{}", res.output);
    Ok(())
}
