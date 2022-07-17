use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use bincode::Decode;
use harness_frontend::{ExecutionError, ExecutionEvent};
use harness_server_types::{ListResponse, Request, RunError, RunMessage, RunRequest};
use harness_types::ConfigId;
use reflection_types::PipelineDescription;

pub fn list(server: &str) -> eyre::Result<ListResponse> {
    decode_from_stream(&mut req(server, Request::List)?).map_err(Into::into)
}

pub struct RunResponse {
    pub exit_code: i32,
    pub output: String,
}

pub fn exec_shader(server: &str, shader: String, metadata: String) -> eyre::Result<RunResponse> {
    exec_shader_with(server, shader, metadata, vec![])
}

#[allow(unused_variables)]
pub fn exec_shader_with(
    server: &str,
    shader: String,
    metadata: String,
    configs: Vec<ConfigId>,
) -> eyre::Result<RunResponse> {
    todo!()
}

pub fn execute(
    server: &str,
    shader: String,
    pipeline_desc: PipelineDescription,
    configs: Vec<ConfigId>,
    timeout: Option<Duration>,
    on_event: &mut dyn FnMut(ExecutionEvent) -> Result<(), ExecutionError>,
) -> Result<(), ExecutionError> {
    let mut stream = req(
        server,
        Request::Run(RunRequest {
            shader,
            pipeline_desc,
            configs,
            timeout,
        }),
    )
    .map_err(|e| ExecutionError::Custom(e.to_string()))?;

    loop {
        match decode_from_stream(&mut stream)? {
            RunMessage::UsingDefaultConfigs(configs) => {
                on_event(ExecutionEvent::UsingDefaultConfigs(configs))?
            }
            RunMessage::ExecStart(config) => on_event(ExecutionEvent::Start(config))?,
            RunMessage::ExecSuccess(buffers) => on_event(ExecutionEvent::Success(buffers))?,
            RunMessage::ExecFailure(stderr) => on_event(ExecutionEvent::Failure(stderr))?,
            RunMessage::ExecTimeout => on_event(ExecutionEvent::Timeout)?,
            RunMessage::End(result) => {
                return result.map_err(|e| match e {
                    RunError::NoDefaultConfigs => ExecutionError::NoDefaultConfigs,
                    RunError::InternalServerError => {
                        ExecutionError::Custom("Internal server error".to_owned())
                    }
                })
            }
        }
    }
}

fn req(server: &str, req: Request) -> eyre::Result<TcpStream> {
    let address = SocketAddr::from_str(server)?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))?;
    bincode::encode_into_std_write(req, &mut stream, bincode::config::standard())?;
    Ok(stream)
}

fn decode_from_stream<T: Decode>(stream: &mut TcpStream) -> Result<T, bincode::error::DecodeError> {
    bincode::decode_from_std_read(stream, bincode::config::standard())
}
