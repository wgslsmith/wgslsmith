pub use ast::writer::Options;

pub fn preprocess(options: Options, shader: &str) -> String {
    let mut buffer = String::new();
    ast::writer::Writer::new(options)
        .write_module(&mut buffer, &parser::parse(shader))
        .unwrap();
    buffer
}
