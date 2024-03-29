use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Parser)]
pub struct Options {
    /// Path to a wgsl shader program (use '-' for stdin).
    #[clap(action, default_value = "-")]
    pub input: String,

    /// Path at which to write output (use '-' for stdout).
    #[clap(short, long, action, default_value = "-")]
    pub output: String,
}

pub fn run(options: Options) -> eyre::Result<()> {
    let source = read_shader_from_path(&options.input)?;
    let ast = parser::parse(&source);

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
        .write_module(&mut Output(output), &ast)
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
