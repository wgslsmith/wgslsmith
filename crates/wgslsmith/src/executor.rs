use std::io::Read;
use std::net::TcpStream;

use clap::Parser;
use harness_server_types::{Request, Response};

pub fn exec_shader(server: &str, shader: String, metadata: String) -> eyre::Result<Response> {
    exec_shader_with(server, shader, metadata, vec![])
}

pub fn exec_shader_with(
    server: &str,
    shader: String,
    metadata: String,
    configs: Vec<String>,
) -> eyre::Result<Response> {
    let mut stream = TcpStream::connect(server).unwrap();

    let req = Request {
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

pub fn run(options: Options) -> eyre::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let res = exec_shader(&options.server, input, options.metadata)?;
    println!("{}", res.output);
    Ok(())
}
