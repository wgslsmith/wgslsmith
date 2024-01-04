use std::fmt::{Display, Result, Write};

use indenter::indented;

use crate::{FnAttr, FnDecl, GlobalConstDecl, GlobalVarDecl, Module, StructDecl};

#[derive(Default)]
pub struct Writer {
    #[allow(unused)]
    options: Options,
}

#[derive(Default)]
pub struct Options {
    pub module_scope_constants: bool,
}

impl Writer {
    pub fn new(options: Options) -> Writer {
        Writer { options }
    }

    pub fn write_module(&self, f: &mut dyn Write, module: &Module) -> Result {
        for decl in &module.structs {
            self.write_struct(f, decl)?;
            writeln!(f)?;
        }

        for decl in &module.consts {
            self.write_global_const(f, decl)?;
            writeln!(f)?;
        }

        for decl in &module.vars {
            self.write_global_var(f, decl)?;
            writeln!(f)?;
        }

        for decl in &module.functions {
            self.write_func(f, decl)?;
            writeln!(f)?;
        }

        Ok(())
    }

    pub fn write_struct(&self, f: &mut dyn Write, decl: &StructDecl) -> Result {
        writeln!(f, "struct {} {{", decl.name)?;

        for member in &decl.members {
            self.write_attrs(&mut indented(f), member.attrs.iter())?;
            writeln!(indented(f), "{}: {},", member.name, member.data_type)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }

    pub fn write_global_const(&self, f: &mut dyn Write, decl: &GlobalConstDecl) -> Result {
        if self.options.module_scope_constants {
            write!(f, "const")?;
        } else {
            write!(f, "let")?;
        }

        writeln!(
            f,
            " {}: {} = {};",
            decl.name, decl.data_type, decl.initializer
        )
    }

    pub fn write_global_var(&self, f: &mut dyn Write, decl: &GlobalVarDecl) -> Result {
        self.write_attrs(f, decl.attrs.iter())?;

        write!(f, "var")?;

        if let Some(qualifier) = &decl.qualifier {
            write!(f, "<{}", qualifier.storage_class)?;
            if let Some(access_mode) = &qualifier.access_mode {
                write!(f, ", {access_mode}")?;
            }
            write!(f, ">")?;
        }

        write!(f, " {}: {}", decl.name, decl.data_type)?;

        if let Some(initializer) = &decl.initializer {
            write!(f, " = {initializer}")?;
        }

        writeln!(f, ";")
    }

    pub fn write_func(&self, f: &mut dyn Write, func: &FnDecl) -> Result {
        for attr in &func.attrs {
            match attr {
                FnAttr::Stage(stage) => {
                    writeln!(f, "@{stage}")?;
                }
                _ => self.write_attr(f, attr)?,
            }
        }

        write!(f, "fn {}(", func.name)?;

        for (i, param) in func.inputs.iter().enumerate() {
            write!(f, "{param}")?;
            if i != func.inputs.len() - 1 {
                f.write_str(", ")?;
            }
        }

        write!(f, ") ")?;

        if let Some(output) = &func.output {
            write!(f, "-> {output} ")?;
        }

        writeln!(f, "{{")?;

        for stmt in &func.body {
            writeln!(indented(f), "{}", stmt)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }

    fn write_attrs<'a, T: Display + 'a>(
        &self,
        f: &mut dyn Write,
        attrs: impl Iterator<Item = &'a T>,
    ) -> Result {
        for attr in attrs {
            self.write_attr(f, attr)?;
        }

        Ok(())
    }

    fn write_attr<T: Display>(&self, f: &mut dyn Write, attr: &T) -> Result {
        writeln!(f, "@{attr}")
    }
}
