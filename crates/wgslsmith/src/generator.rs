mod cx;
mod expr;
mod fns;
mod scope;
mod stmt;
mod structs;
mod utils;

use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentOp, AssignmentStatement, Expr, ExprNode, FnAttr, FnDecl,
    GlobalVarAttr, GlobalVarDecl, LetDeclStatement, Module, Postfix, ShaderStage, StorageClass,
    VarQualifier,
};
use rand::prelude::StdRng;
use rand::Rng;

use crate::generator::scope::Scope;
use crate::Options;

use self::cx::Context;
use self::structs::StructKind;

pub struct Generator<'a> {
    rng: &'a mut StdRng,
    options: Rc<Options>,
    cx: Context,
    block_depth: u32,
    expression_depth: u32,
    return_type: Option<DataType>,
    scope: Scope,
}

impl<'a> Generator<'a> {
    pub fn new(rng: &'a mut StdRng, options: Rc<Options>) -> Self {
        Generator {
            rng,
            options: options.clone(),
            cx: Context::new(options),
            block_depth: 0,
            expression_depth: 0,
            return_type: None,
            scope: Scope::empty(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn gen_module(&mut self) -> Module {
        let struct_count = self
            .rng
            .gen_range(self.options.min_structs..=self.options.max_structs);

        for i in 1..=struct_count {
            let name = format!("Struct_{}", i);
            let decl = self.gen_struct(name);
            self.cx.types.insert(decl);
        }

        let ub_type_decl =
            self.gen_struct_with("UniformBuffer".to_owned(), StructKind::HostShareable);
        let sb_type_decl =
            self.gen_struct_with("StorageBuffer".to_owned(), StructKind::HostShareable);

        self.scope
            .insert_readonly("u_input".to_owned(), DataType::Struct(ub_type_decl.clone()));

        let mut global_vars = vec![
            GlobalVarDecl {
                attrs: vec![GlobalVarAttr::Group(0), GlobalVarAttr::Binding(0)],
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Uniform,
                    access_mode: None,
                }),
                name: "u_input".to_owned(),
                data_type: DataType::Struct(ub_type_decl.clone()),
                initializer: None,
            },
            GlobalVarDecl {
                attrs: vec![GlobalVarAttr::Group(0), GlobalVarAttr::Binding(1)],
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Storage,
                    access_mode: Some(AccessMode::ReadWrite),
                }),
                name: "s_output".to_owned(),
                data_type: DataType::Struct(sb_type_decl.clone()),
                initializer: None,
            },
        ];

        for i in 0..self.rng.gen_range(0..=5) {
            let name = format!("global{i}");
            global_vars.push(self.gen_global_var(name));
        }

        let entrypoint = self.gen_entrypoint_function(
            DataType::Struct(ub_type_decl.clone()),
            DataType::Struct(sb_type_decl.clone()),
        );

        let Context { types, fns } =
            std::mem::replace(&mut self.cx, Context::new(self.options.clone()));

        let mut functions = fns.into_fns();

        functions.push(entrypoint);

        Module {
            structs: {
                let mut structs = types.into_structs();
                structs.push(ub_type_decl);
                structs.push(sb_type_decl);
                structs
            },
            consts: vec![],
            vars: global_vars,
            functions,
        }
    }

    fn gen_global_var(&mut self, name: String) -> GlobalVarDecl {
        let mut data_type = self.cx.types.select(self.rng);

        if self.rng.gen_bool(0.5) {
            data_type = DataType::Array(Rc::new(data_type), Some(self.rng.gen_range(1..=32)));
        }

        self.scope.insert_mutable(name.clone(), data_type.clone());

        // TODO: Enable intiialisers for arrays after https://github.com/gfx-rs/naga/pull/1914 is merged
        let initializer = if !matches!(data_type, DataType::Array(_, _)) && self.rng.gen_bool(0.5) {
            Some(self.gen_const_expr(&data_type))
        } else {
            None
        };

        GlobalVarDecl {
            attrs: vec![],
            qualifier: Some(VarQualifier {
                storage_class: StorageClass::Private,
                access_mode: None,
            }),
            name,
            data_type,
            initializer,
        }
    }

    #[tracing::instrument(skip(self))]
    fn gen_entrypoint_function(&mut self, in_buf_type: DataType, out_buf_type: DataType) -> FnDecl {
        let stmt_count = self.rng.gen_range(5..10);
        let (scope, mut block) = self.gen_stmt_block(stmt_count);

        block.push(
            LetDeclStatement::new(
                "x".to_owned(),
                ExprNode {
                    data_type: DataType::Scalar(ScalarType::U32),
                    expr: Expr::Postfix(
                        Box::new(ExprNode {
                            data_type: in_buf_type,
                            expr: Expr::Var("u_input".to_owned()),
                        }),
                        Postfix::Member("a".to_owned()),
                    ),
                },
            )
            .into(),
        );

        self.with_scope(scope, |this| {
            block.push(
                AssignmentStatement::new(
                    AssignmentLhs::name("s_output".to_owned(), out_buf_type.clone()),
                    AssignmentOp::Simple,
                    this.gen_expr(&out_buf_type),
                )
                .into(),
            );
        });

        FnDecl {
            attrs: vec![
                FnAttr::Stage(ShaderStage::Compute),
                FnAttr::WorkgroupSize(1),
            ],
            name: "main".to_owned(),
            inputs: vec![],
            output: None,
            body: block,
        }
    }

    fn with_scope<T>(&mut self, scope: Scope, block: impl FnOnce(&mut Self) -> T) -> (Scope, T) {
        let old_scope = std::mem::replace(&mut self.scope, scope);
        let res = block(self);
        (std::mem::replace(&mut self.scope, old_scope), res)
    }
}
