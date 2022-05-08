use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};

use clap::StructOpt;
use threadpool::ThreadPool;

#[derive(clap::Parser)]
struct Options {
    /// Server bind address.
    #[clap(short, long, default_value = "localhost:0")]
    address: String,

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

    let listener = TcpListener::bind(options.address).unwrap();
    let address = listener.local_addr().unwrap();
    println!("Server listening at {address}");

    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let harness_path = Box::leak(exe_dir.join("harness").into_boxed_path());

    if !harness_path.exists() {
        eprintln!(
            "Error: harness executable not found at `{}`",
            harness_path.display()
        );
    } else {
        println!("Using harness executable at {}", harness_path.display());
    }

    for stream in listener.incoming() {
        let harness_path = &*harness_path;
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
            let mut harness = Command::new(harness_path)
                .args(["run", "-", metadata.trim()])
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
