use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Parser)]
pub struct Options {
    // We still need input and output
    #[clap(action, default_value = "-")]
    pub input: String,

    #[clap(action, default_value = "-")]
    pub output: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub workgroup_count: u32,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub dispatch_size: u32,
}

pub fn run(options: Options) -> eyre::Result<()> {
    eprintln!(
        "Changing shader to run on {} threads in {} blocks...",
        options.workgroup_count,
        options.dispatch_size,
    );

    let shader = read_shader_from_path(&options.input)?;
    let ast = parser::parse(&shader);

    // Toss the work to our library, then we write after
    // Not sure if we can change the block size here
    let result = crate::thread(ast, options.workgroup_count, options.dispatch_size);

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
