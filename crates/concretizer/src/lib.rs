mod builtin;
pub mod concretizer;
mod helper;
pub mod value;

use crate::concretizer::*;
use ast::*;

pub fn concretize(ast: Module) -> Module {
    concretize_with(ast, Options::default())
}

pub fn concretize_with(mut ast: Module, options: Options) -> Module {
    let mut concretizer = Concretizer::new(options);

    let functions = ast
        .functions
        .into_iter()
        .map(|f| concretizer.concretize_fn(f))
        .collect::<Vec<_>>();

    ast.functions = functions;

    ast
}
