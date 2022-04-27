use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

fn main() {
    let server = std::env::args().nth(1).unwrap();
    let metadata = std::env::args().nth(2).unwrap();

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    let stream = TcpStream::connect(server).unwrap();
    let mut writer = BufWriter::new(&stream);

    writeln!(writer, "{metadata}").unwrap();
    writeln!(writer, "{}", input.len()).unwrap();
    writeln!(writer, "{input}").unwrap();

    writer.flush().unwrap();

    let mut buf = String::new();
    BufReader::new(&stream).read_line(&mut buf).unwrap();

    std::process::exit(buf.trim().parse().unwrap());
}
