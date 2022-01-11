use std::fmt::{Display, Write};

use indenter::indented;

use crate::types::DataType;

#[derive(Debug, PartialEq, Eq)]
pub struct StructMember {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StructDecl {
    pub name: String,
    pub members: Vec<StructMember>,
}

impl Display for StructDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;

        for member in &self.members {
            writeln!(indented(f), "{}: {};", member.name, member.data_type)?;
        }

        writeln!(f, "}};")?;

        Ok(())
    }
}
