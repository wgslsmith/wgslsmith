use std::io::Read;
use std::net::TcpStream;

use clap::Parser;

#[derive(Debug, bincode::Encode)]
struct Request<'a> {
    shader: &'a str,
    metadata: &'a str,
    configs: Vec<&'a str>,
}

#[derive(Debug, bincode::Decode)]
pub struct Response {
    pub exit_code: i32,
    pub output: String,
}

pub fn exec_shader(server: &str, shader: &str, metadata: &str) -> eyre::Result<Response> {
    exec_shader_with(server, shader, metadata, vec![])
}

pub fn exec_shader_with(
    server: &str,
    shader: &str,
    metadata: &str,
    configs: Vec<&str>,
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
    let res = exec_shader(&options.server, &input, &options.metadata)?;
    println!("{}", res.output);
    Ok(())
}
