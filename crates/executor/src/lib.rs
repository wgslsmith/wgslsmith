use std::net::TcpStream;

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

pub fn exec_shader(server: &str, shader: &str, metadata: &str) -> Response {
    exec_shader_with(server, shader, metadata, vec![])
}

pub fn exec_shader_with(
    server: &str,
    shader: &str,
    metadata: &str,
    configs: Vec<&str>,
) -> Response {
    let mut stream = TcpStream::connect(server).unwrap();

    let req = Request {
        shader,
        metadata,
        configs,
    };

    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard()).unwrap();
    bincode::decode_from_std_read(&mut stream, bincode::config::standard()).unwrap()
}
