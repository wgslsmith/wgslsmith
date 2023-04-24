use std::io::{self, BufReader, BufWriter};
use std::net::TcpListener;

use clap::Parser;
use color_eyre::eyre::{self, eyre};
use frontend::{ExecutionError, ExecutionEvent};
use server_types::{ListResponse, Request, RunError, RunMessage, RunRequest};
use threadpool::ThreadPool;

use crate::HarnessHost;

#[derive(Parser)]
pub struct Options {
    /// Server bind address.
    #[clap(short, long, action, default_value = "localhost:0")]
    address: String,

    /// Number of worker threads to use for running shaders in parallel.
    ///
    /// Defaults to the number of available CPUs.
    #[clap(long, action)]
    parallelism: Option<usize>,
}

pub fn run<Host: HarnessHost>(options: Options) -> eyre::Result<()> {
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

            let req =
                bincode::decode_from_std_read(&mut reader, bincode::config::standard()).unwrap();

            let writer = BufWriter::new(&stream);
            match req {
                Request::List => handle_list_request(writer).unwrap(),
                Request::Run(req) => handle_run_request::<Host, _>(req, writer).unwrap(),
            }
        });
    }

    Ok(())
}

fn handle_list_request(mut writer: impl io::Write) -> eyre::Result<()> {
    let configs = crate::query_configs();
    let res = ListResponse { configs };
    send(&mut writer, res)?;
    Ok(())
}

fn handle_run_request<Host: HarnessHost, W: io::Write>(
    req: RunRequest,
    mut writer: W,
) -> eyre::Result<()> {
    let on_event = |e| {
        let message = match e {
            ExecutionEvent::UsingDefaultConfigs(configs) => {
                RunMessage::UsingDefaultConfigs(configs)
            }
            ExecutionEvent::Start(config) => RunMessage::ExecStart(config),
            ExecutionEvent::Success(buffers, flow) => RunMessage::ExecSuccess(buffers, flow),
            ExecutionEvent::Failure(stderr) => RunMessage::ExecFailure(stderr),
            ExecutionEvent::Timeout => RunMessage::ExecTimeout,
        };
        send(&mut writer, message)?;
        writer.flush()?;
        Ok(())
    };

    let result = crate::execute::<Host, _>(
        &req.shader,
        req.workgroups,
        req.flow,
        &req.pipeline_desc,
        &req.configs,
        req.timeout,
        on_event,
    )
    .map_err(|e| match e {
        ExecutionError::NoDefaultConfigs => RunError::NoDefaultConfigs,
        e => {
            eprintln!("{:?}", eyre!(e));
            RunError::InternalServerError
        }
    });

    send(&mut writer, RunMessage::End(result))?;

    Ok(())
}

fn send(
    writer: &mut impl io::Write,
    val: impl bincode::Encode,
) -> Result<(), bincode::error::EncodeError> {
    bincode::encode_into_std_write(val, writer, bincode::config::standard())?;
    Ok(())
}
