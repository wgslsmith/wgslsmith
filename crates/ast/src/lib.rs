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

#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub structs: Vec<Rc<StructDecl>>,
    pub consts: Vec<GlobalConstDecl>,
    pub vars: Vec<GlobalVarDecl>,
    pub functions: Vec<FnDecl>,
}

struct FmtArgs<'a>(&'a [ExprNode]);

impl<'a> std::fmt::Display for FmtArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, e) in self.0.iter().enumerate() {
            e.fmt(f)?;
            if i != self.0.len() - 1 {
                f.write_str(", ")?;
            }
        }

        Ok(())
    }
}
