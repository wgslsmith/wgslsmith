use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Parser)]
pub struct Options {
    /// Path to a wgsl shader program (use '-' for stdin).
    #[clap(default_value = "-")]
    input: String,

    /// Path at which to write output (use '-' for stdout).
    #[clap(default_value = "-")]
    output: String,
}

pub fn run(options: Options) -> eyre::Result<()> {
    let input = read_shader_from_path(&options.input)?;
    let ast = parser::parse(&input);

    let result = reconditioner::analysis::analyse(&ast);
    if !result {
        eprintln!("rejecting due to possible invalid aliasing");
        std::process::exit(1);
    }

    let result = reconditioner::recondition(ast);

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
