use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpListener;
use std::process::Stdio;

use clap::StructOpt;
use threadpool::ThreadPool;

#[derive(clap::Parser)]
struct Options {
    /// Server host address.
    #[clap(short = 'b', long, default_value = "localhost")]
    host: String,

    /// Server port.
    #[clap(short, long, default_value = "0")]
    port: u16,

    /// Number of worker threads to use for running shaders in parallel.
    ///
    /// Defaults to the number of available CPUs.
    #[clap(long)]
    parallelism: Option<usize>,
}

fn main() {
    let options = Options::parse();

    let parallelism = options
        .parallelism
        .unwrap_or_else(|| std::thread::available_parallelism().unwrap().get());

    let pool = ThreadPool::new(parallelism);
    println!("Using thread pool with {parallelism} threads");

    let address = (options.host.as_str(), options.port);
    let listener = TcpListener::bind(address).unwrap();

    let address = listener.local_addr().unwrap();
    println!("Server listening at {address}");

    for stream in listener.incoming() {
        pool.execute(move || {
            let stream = stream.unwrap();

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let mut metadata = String::new();
            reader.read_line(&mut metadata).unwrap();

            let mut buf = String::new();
            reader.read_line(&mut buf).unwrap();

            let total_bytes: usize = buf.trim_end().parse().unwrap();
            println!("reading {total_bytes} bytes from stream");

            let mut buf = vec![0; total_bytes];
            reader.read_exact(buf.as_mut()).unwrap();

            let shader = String::from_utf8(buf).unwrap();

            println!("executing harness");
            let mut harness = std::process::Command::new("harness")
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
