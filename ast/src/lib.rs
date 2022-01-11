pub mod types;

mod expr;
mod func;
mod globals;
mod stmt;
mod structs;

pub use expr::*;
pub use func::*;
pub use globals::*;
pub use stmt::*;
pub use structs::*;

use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub structs: Vec<StructDecl>,
    pub vars: Vec<GlobalVarDecl>,
    pub functions: Vec<FnDecl>,
    pub entrypoint: FnDecl,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for decl in &self.structs {
            writeln!(f, "{}", decl)?;
        }

        for decl in &self.vars {
            writeln!(f, "{}", decl)?;
        }

        self.entrypoint.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttrList<T>(pub Vec<T>);

impl<T> FromIterator<T> for AttrList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        AttrList(Vec::from_iter(iter))
    }
}

impl<T: Display> Display for AttrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.0.is_empty() {
            write!(f, "[[")?;

            for (i, attr) in self.0.iter().enumerate() {
                write!(f, "{}", attr)?;
                if i != self.0.len() - 1 {
                    write!(f, ", ")?;
                }
            }

            write!(f, "]]")?;
        }

        Ok(())
    }
}
