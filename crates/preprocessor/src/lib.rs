pub use ast::writer::Options;

pub fn preprocess(options: Options, mut shader: String) -> String {
    if options.module_scope_constants {
        panic!("module scope constants are not supported yet");
    }

    shader
}
