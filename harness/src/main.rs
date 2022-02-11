mod dawn;
mod wgpu;

use std::io::Read;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let shader = read_shader_from_stdin()?;

    // println!("----- BEGIN SHADER -----");
    // print!("{}", shader);
    // println!("----- END SHADER -------");

    let out_1 = futures::executor::block_on(wgpu::run(&shader))?;
    let out_2 = futures::executor::block_on(dawn::run(&shader))?;

    for (i, (v1, v2)) in out_1.0.into_iter().zip(out_2.0.into_iter()).enumerate() {
        if v1 == v2 {
            println!("output[{}]: {} == {}", i, v1, v2);
        } else {
            println!("output[{}]: {} != {}", i, v1, v2);
        }
    }

    Ok(())
}

fn read_shader_from_stdin() -> Result<String> {
    let mut shader = String::new();
    std::io::stdin().read_to_string(&mut shader)?;
    Ok(shader)
}

#[repr(C)]
#[derive(Debug)]
pub struct Buffer<const N: usize>([u32; N]);

impl<const N: usize> Buffer<N> {
    const SIZE: usize = std::mem::size_of::<Self>();

    pub fn from_bytes(slice: &[u8]) -> Self {
        let values = slice
            .chunks_exact(4)
            .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));

        let mut buf = Buffer([0; N]);
        for (i, v) in values.enumerate() {
            if i > N {
                break;
            }
            buf.0[i] = v;
        }

        buf
    }
}
