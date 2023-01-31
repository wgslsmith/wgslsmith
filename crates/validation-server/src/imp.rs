use std::io::{BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{env, ptr};

use bincode::Encode;
use clap::Parser;
use color_eyre::eyre;
use tempfile::NamedTempFile;
use threadpool::ThreadPool;
use types::{GetCountResponse, Request, ValidateResponse};
use windows::core::PCSTR;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;

#[derive(Parser)]
pub struct Options {
    /// Server bind address.
    #[clap(short, long, action, default_value = "localhost:0")]
    address: String,

    /// Number of worker threads to use.
    ///
    /// Defaults to the number of available CPUs.
    #[clap(long, action)]
    parallelism: Option<usize>,

    #[clap(short, long, action)]
    quiet: bool,
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

    let quiet = options.quiet;
    let counter = Arc::new(AtomicU64::new(0));

    for stream in listener.incoming() {
        let counter = counter.clone();
        counter.fetch_add(1, Ordering::SeqCst);
        pool.execute(move || {
            let stream = stream.unwrap();

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let req: Request =
                bincode::decode_from_std_read(&mut reader, bincode::config::standard()).unwrap();

            enum Response {
                GetCount(GetCountResponse),
                Validate(ValidateResponse),
            }

            impl Encode for Response {
                fn encode<E: bincode::enc::Encoder>(
                    &self,
                    encoder: &mut E,
                ) -> Result<(), bincode::error::EncodeError> {
                    match self {
                        Response::GetCount(inner) => inner.encode(encoder),
                        Response::Validate(inner) => inner.encode(encoder),
                    }
                }
            }

            let res = match req {
                Request::GetCount => Response::GetCount(GetCountResponse {
                    count: counter.load(Ordering::SeqCst),
                }),
                Request::ResetCount => {
                    counter.store(0, Ordering::SeqCst);
                    return;
                }
                Request::Validate { backend, source } => match backend {
                    types::Backend::Hlsl => {
                        Response::Validate(validate_hlsl(&source, quiet).unwrap())
                    }
                    types::Backend::Msl => {
                        Response::Validate(validate_metal(&source, quiet).unwrap())
                    }
                },
            };

            bincode::encode_into_std_write(res, &mut writer, bincode::config::standard()).unwrap();
        });
    }

    Ok(())
}

fn validate_hlsl(hlsl: &str, quiet: bool) -> eyre::Result<ValidateResponse> {
    unsafe {
        let mut error_messages = None;

        let result = D3DCompile(
            hlsl.as_ptr() as _,
            hlsl.len(),
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

        if result.is_err() {
            let blob = error_messages.unwrap();
            let ptr = blob.GetBufferPointer();
            let size = blob.GetBufferSize();
            let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
            let messages = String::from_utf8(slice.to_owned())?;
            if !quiet {
                println!("{messages}");
            }
            return Ok(ValidateResponse::Failure(messages));
        }
    }

    Ok(ValidateResponse::Success)
}

fn validate_metal(metal: &str, quiet: bool) -> eyre::Result<ValidateResponse> {
    let mut file = NamedTempFile::new_in(env::current_dir()?)?;
    write!(file, "{metal}")?;
    file.flush()?;

    let output = Command::new("./Metal Developer Tools/macos/bin/metal.exe")
        .args(["-x", "metal"])
        .args(["-o", "NUL"])
        .arg("-std=osx-metal2.0")
        .arg("-c")
        .arg(file.path())
        .args([""])
        .stderr(Stdio::piped())
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;

    if output.status.code().unwrap() != 0 {
        if !quiet {
            println!("{stderr}");
        }
        return Ok(ValidateResponse::Failure(stderr));
    }

    Ok(ValidateResponse::Success)
}
