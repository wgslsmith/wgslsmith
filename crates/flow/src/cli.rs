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

    // TODO: Potenially we want options for flow add here
    let result = crate::flow(ast);

    struct Output(Box<dyn std::io::Write>);

    impl std::fmt::Write for Output {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            use std::io::Write;
            self.0.write_all(s.as_bytes()).unwrap();
            Ok(())
        }
    }

    let output: Box<dyn std::io::Write> = match options.output.as_str() {
        "-" => Box::new(std::io::stdout()),
        path => Box::new(File::create(path)?),
    };

    ast::writer::Writer::default()
        .write_module(&mut Output(output), &result)
        .unwrap();

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
