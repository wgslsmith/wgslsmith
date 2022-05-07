use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;

pub fn exec_shader(server: &str, shader: &str, metadata: &str) -> i32 {
    let stream = TcpStream::connect(server).unwrap();
    let mut writer = BufWriter::new(&stream);

    writeln!(writer, "{metadata}").unwrap();
    writeln!(writer, "{}", shader.len()).unwrap();
    writeln!(writer, "{shader}").unwrap();

    writer.flush().unwrap();

    let mut buf = String::new();
    BufReader::new(&stream).read_line(&mut buf).unwrap();

    buf.trim().parse().unwrap()
}
