use std::fs::File;
use std::io::Read;

use clap::{Parser, ValueEnum};

// use crate::analysis;

#[derive(Parser)]
pub struct Options {
    // Path to a wgsl program
    #[clap(action, default_value = "-")]
    pub input: String,

    // Path to write output
    #[clap(action, default_value = "-")]
    pub output: String,

    #[clap(
        long,
        value_enum,
        action,
        use_value_delimiter(true),
        require_value_delimiter(true)
    )]
    pub enable: Vec<Feature>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Feature {
    PlaceHolder, // This is a carry over from the original reconditioner
}

pub fn run(options: Options) -> eyre::Result<()> {
    let input = read_shader_from_path(&options.input)?;
    let ast = parser::parse(&input);

    let result = crate::flow(ast);

    //println!("{}", input);
    //let ast = parser::parse(&input);
    Ok(())
}

fn read_shader_from_path(path: &str) -> eyre::Result<String> {
    let mut input: Box<dyn Read> = match path {
        "-" => Box::new(std::io::stdin()),
        path => Box::new(File::open(path)?),
    };

    let mut shader = String::new();
    input.read_to_string(&mut shader)?;

    Ok(shader)
}
