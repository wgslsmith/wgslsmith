mod dawn;
mod wgpu;

use color_eyre::Result;
use common::ShaderMetadata;

pub struct Execution {
    pub dawn: Vec<Vec<u8>>,
    pub wgpu: Vec<Vec<u8>>,
}

pub fn execute(shader: &str, meta: &ShaderMetadata) -> Result<Execution> {
    let out_1 = futures::executor::block_on(dawn::run(shader, meta))?;
    let out_2 = futures::executor::block_on(wgpu::run(shader, meta))?;

    Ok(Execution {
        dawn: out_1,
        wgpu: out_2,
    })
}
