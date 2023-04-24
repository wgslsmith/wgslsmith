use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use bincode::Decode;
use eyre::{eyre, Context};
use harness_frontend::{ExecutionError, ExecutionEvent};
use harness_server_types::{ListResponse, Request, RunError, RunMessage, RunRequest};
use harness_types::ConfigId;
use reflection_types::PipelineDescription;

pub fn list(server: &str) -> eyre::Result<ListResponse> {
    decode_from_stream(&mut req(server, Request::List)?).map_err(Into::into)
}

pub fn execute(
    server: &str,
    shader: String,
    workgroups: u32,
    flow: bool,
    pipeline_desc: PipelineDescription,
    configs: Vec<ConfigId>,
    timeout: Option<Duration>,
    on_event: &mut dyn FnMut(ExecutionEvent) -> Result<(), ExecutionError>,
) -> Result<(), ExecutionError> {
    let mut stream = req(
        server,
        Request::Run(RunRequest {
            shader,
            workgroups,
            flow,
            pipeline_desc,
            configs,
            timeout,
        }),
    )?;

    loop {
        match decode_from_stream(&mut stream)? {
            RunMessage::UsingDefaultConfigs(configs) => {
                on_event(ExecutionEvent::UsingDefaultConfigs(configs))?
            }
            RunMessage::ExecStart(config) => on_event(ExecutionEvent::Start(config))?,
            RunMessage::ExecSuccess(buffers, flow) => {
                on_event(ExecutionEvent::Success(buffers, flow))?
            }
            RunMessage::ExecFailure(stderr) => on_event(ExecutionEvent::Failure(stderr))?,
            RunMessage::ExecTimeout => on_event(ExecutionEvent::Timeout)?,
            RunMessage::End(result) => {
                return result.map_err(|e| match e {
                    RunError::NoDefaultConfigs => ExecutionError::NoDefaultConfigs,
                    RunError::InternalServerError => {
                        ExecutionError::Other(eyre!("internal server error"))
                    }
                })
            }
        }
    }
}

fn req(server: &str, req: Request) -> eyre::Result<TcpStream> {
    let address = SocketAddr::from_str(server)?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))
        .wrap_err_with(|| format!("failed to connect to {server}"))?;
    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard())?;
    Ok(stream)
}

fn decode_from_stream<T: Decode>(stream: &mut TcpStream) -> Result<T, bincode::error::DecodeError> {
    bincode::decode_from_std_read(stream, bincode::config::standard())
}
