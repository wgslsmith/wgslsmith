use std::fmt::Write as _;
use std::io::{BufRead, BufReader, BufWriter, Write as _};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Stdio};

use clap::Parser;
use color_eyre::eyre;
use threadpool::ThreadPool;

#[derive(Debug, bincode::Decode)]
struct Request {
    shader: String,
    metadata: String,
    configs: Vec<String>,
}

#[derive(Debug, bincode::Encode)]
struct Response {
    exit_code: i32,
    output: String,
}

#[derive(Parser)]
pub struct Options {
    /// Server bind address.
    #[clap(short, long, default_value = "localhost:0")]
    address: String,

    /// Number of worker threads to use for running shaders in parallel.
    ///
    /// Defaults to the number of available CPUs.
    #[clap(long)]
    parallelism: Option<usize>,
}

pub fn run(options: Options) -> eyre::Result<()> {
    let parallelism = options
        .parallelism
        .unwrap_or_else(|| std::thread::available_parallelism().unwrap().get());

    let pool = ThreadPool::new(parallelism);
    println!("Using thread pool with {parallelism} threads");

    let listener = TcpListener::bind(options.address).unwrap();
    let address = listener.local_addr().unwrap();
    println!("Server listening at {address}");

    let harness_path = Box::leak(std::env::current_exe().unwrap().into_boxed_path());
    let harness_path = &*harness_path;

    for stream in listener.incoming() {
        pool.execute(move || {
            let stream = stream.unwrap();

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let req: Request =
                bincode::decode_from_std_read(&mut reader, bincode::config::standard()).unwrap();

            println!(">> starting harness");

            let res = exec_harness(harness_path, &req);

            println!(">> harness exited with {}", res.exit_code);

            bincode::encode_into_std_write(res, &mut writer, bincode::config::standard()).unwrap();
        });
    }

    Ok(())
}

fn exec_harness(path: &Path, req: &Request) -> Response {
    let mut cmd = Command::new(path);

    cmd.args(["run", "-", &req.metadata]);

    for config in &req.configs {
        cmd.args(["-c", config]);
    }

    let mut harness = cmd
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let mut stdin = harness.stdin.take().unwrap();
        stdin.write_all(req.shader.as_bytes()).unwrap();
    }

    let stderr = {
        let stderr = harness.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let mut buf = String::new();

        for line in reader.lines().flatten() {
            eprintln!("{line}");
            writeln!(&mut buf, "{line}").unwrap();
        }

        buf
    };

    let status = harness.wait().unwrap();
    let exit_code = status.code().unwrap();

    Response {
        exit_code,
        output: stderr,
    }
}
