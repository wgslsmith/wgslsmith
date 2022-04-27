use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpListener;
use std::process::Stdio;

use threadpool::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let parallelism = std::thread::available_parallelism().unwrap().get();
    let pool = ThreadPool::new(parallelism);

    println!("Using thread pool with {parallelism} threads");

    for stream in listener.incoming() {
        pool.execute(move || {
            let stream = stream.unwrap();

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let mut metadata = String::new();
            reader.read_line(&mut metadata).unwrap();

            let mut buf = String::new();
            reader.read_line(&mut buf).unwrap();
            println!("{buf}");

            let total_bytes: usize = buf.trim_end().parse().unwrap();
            println!("reading {total_bytes} bytes from stream");

            let mut buf = vec![0; total_bytes];
            reader.read_exact(buf.as_mut()).unwrap();

            let shader = String::from_utf8(buf).unwrap();

            println!("executing harness");
            let mut harness = std::process::Command::new("harness.exe")
                .arg("--metadata")
                .arg(metadata.trim())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();

            {
                let mut stdin = harness.stdin.take().unwrap();
                stdin.write_all(shader.as_bytes()).unwrap();
            }

            let status = harness.wait().unwrap();
            let exit_code = status.code().unwrap();

            println!(">> exited with {exit_code}");

            writeln!(writer, "{exit_code}").unwrap();
            writer.flush().unwrap();
        });
    }
}
