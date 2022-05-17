pub mod types;
pub mod writer;

mod builtins;
mod expr;
mod func;
mod globals;
mod stmt;
mod structs;

pub use builtins::*;
pub use expr::*;
pub use func::*;
pub use globals::*;
pub use stmt::*;
pub use structs::*;

use std::rc::Rc;

pub use types::{DataType, ScalarType};

#[derive(Debug, PartialEq)]
pub struct Module {
    pub structs: Vec<Rc<StructDecl>>,
    pub consts: Vec<GlobalConstDecl>,
    pub vars: Vec<GlobalVarDecl>,
    pub functions: Vec<FnDecl>,
}
