mod dawn;
mod wgpu;

use std::fs::File;
use std::io::Read;

use clap::Parser;
use color_eyre::Result;
use common::ShaderMetadata;

#[derive(Parser)]
struct Options {
    /// Path to wgsl shader program to be executed (use '-' for stdin)
    #[clap(default_value_t = String::from("-"))]
    input: String,
}

fn main() -> Result<()> {
    let options = Options::parse();

    color_eyre::install()?;
    env_logger::init();

    let input = read_shader_from_path(&options.input)?;
    let (meta, shader) = input.split_once('\n').unwrap();
    let meta: ShaderMetadata = meta
        .strip_prefix("//")
        .map(|it| it.trim_start())
        .and_then(|it| serde_json::from_str(it).ok())
        .expect("first line of shader must be a comment containing json metadata");

    // println!("----- BEGIN SHADER -----");
    // print!("{}", shader);
    // println!("----- END SHADER -------");

    let out_1 = futures::executor::block_on(dawn::run(shader, &meta))?;
    let out_2 = futures::executor::block_on(wgpu::run(shader, &meta))?;

    println!("========== Results ==========");
    println!("dawn: result={:x?}", out_1);
    println!("wgpu: result={:x?}", out_2);

    if out_1 != out_2 {
        println!("mismatch!");
        std::process::exit(1);
    }

    Ok(())
}

fn read_shader_from_path(path: &str) -> Result<String> {
    let mut input: Box<dyn Read> = match path {
        "-" => Box::new(std::io::stdin()),
        path => Box::new(File::open(path)?),
    };

    let mut shader = String::new();
    input.read_to_string(&mut shader)?;

    Ok(shader)
}
