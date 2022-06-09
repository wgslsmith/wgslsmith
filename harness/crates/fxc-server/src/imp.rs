use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::ptr;
use std::time::Instant;

use clap::Parser;
use color_eyre::eyre;
use threadpool::ThreadPool;
use windows::core::PCSTR;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;

#[derive(Debug, bincode::Decode)]
struct Request {
    hlsl: String,
}

#[derive(Debug, bincode::Encode)]
enum Response {
    Success,
    Failure(String),
}

#[derive(Parser)]
pub struct Options {
    /// Server bind address.
    #[clap(short, long, default_value = "localhost:0")]
    address: String,

    /// Number of worker threads to use.
    ///
    /// Defaults to the number of available CPUs.
    #[clap(long)]
    parallelism: Option<usize>,
}

pub fn run() -> eyre::Result<()> {
    let options = Options::parse();
    let parallelism = options
        .parallelism
        .unwrap_or_else(|| std::thread::available_parallelism().unwrap().get());

    let pool = ThreadPool::new(parallelism);
    println!("Using thread pool with {parallelism} threads");

    let listener = TcpListener::bind(options.address).unwrap();
    let address = listener.local_addr().unwrap();
    println!("Server listening at {address}");

    for stream in listener.incoming() {
        pool.execute(move || {
            let stream = stream.unwrap();

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let req: Request =
                bincode::decode_from_std_read(&mut reader, bincode::config::standard()).unwrap();

            let res = validate_hlsl(&req).unwrap();

            bincode::encode_into_std_write(res, &mut writer, bincode::config::standard()).unwrap();
        });
    }

    Ok(())
}

fn validate_hlsl(req: &Request) -> eyre::Result<Response> {
    unsafe {
        let mut error_messages = None;

        let start = Instant::now();

        let result = D3DCompile(
            req.hlsl.as_ptr() as _,
            req.hlsl.len(),
            None,
            ptr::null(),
            None,
            PCSTR("main\0".as_ptr()),
            PCSTR("cs_5_1\0".as_ptr()),
            0,
            0,
            &mut None,
            &mut error_messages,
        );

        let elapsed = Instant::now() - start;

        println!("Compilation took {}s", elapsed.as_secs_f64());

        if result.is_err() {
            let blob = error_messages.unwrap();
            let ptr = blob.GetBufferPointer();
            let size = blob.GetBufferSize();
            let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
            let messages = String::from_utf8(slice.to_owned())?;
            println!("{messages}");
            return Ok(Response::Failure(messages));
        }
    }

    Ok(Response::Success)
}
