mod cx;
mod expr;
mod fns;
mod scope;
mod stmt;
mod structs;
mod utils;

pub mod builtins;

use std::rc::Rc;

use ast::types::{DataType, MemoryViewType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentOp, AssignmentStatement, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, LetDeclStatement, Module, Postfix, PostfixExpr, ShaderStage, Statement,
    StorageClass, VarExpr, VarQualifier,
};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;
use rand_distr::{Binomial, Distribution, StandardNormal};

use crate::gen::scope::Scope;
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
    global_scope: Scope,
    scope: Scope,
    current_block: Vec<Statement>,
    f32_dist: StandardNormal,
    i32_dist: Binomial,
    u32_dist: Binomial,
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
            global_scope: Scope::empty(),
            scope: Scope::empty(),
            current_block: vec![],
            f32_dist: StandardNormal,
            i32_dist: Binomial::new(i32::MAX as u64 * 2, 0.5)
                .expect("failed to create binomial distribution"),
            u32_dist: Binomial::new(u32::MAX as u64 * 2, 0.5)
                .expect("failed to create binomial distribution"),
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
            self.gen_struct_with("UniformBuffer".to_owned(), StructKind::UniformBuffer);
        let sb_type_decl =
            self.gen_struct_with("StorageBuffer".to_owned(), StructKind::HostShareable);

        self.global_scope
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

        let mem_view = MemoryViewType::new(data_type.clone(), StorageClass::Private);
        let ref_type = DataType::Ref(mem_view);

        self.global_scope.insert_mutable(name.clone(), ref_type);

        let initializer = if self.rng.gen_bool(0.5) {
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
        let (_, block) = self.with_scope(self.global_scope.clone(), |this| {
            let (scope, block) = this.gen_stmt_block(stmt_count);

            let prev_block = std::mem::replace(&mut this.current_block, block);

            this.with_scope(scope, |this| {
                this.current_block.push(
                    LetDeclStatement::new(
                        "x",
                        PostfixExpr::new(
                            VarExpr::new("u_input").into_node(in_buf_type),
                            Postfix::member("a"),
                        ),
                    )
                    .into(),
                );

                let out_lhs = AssignmentLhs::name("s_output", out_buf_type.clone());
                let out_rhs = this.gen_expr(&out_buf_type);
                this.current_block
                    .push(AssignmentStatement::new(out_lhs, AssignmentOp::Simple, out_rhs).into());
            });

            std::mem::replace(&mut this.current_block, prev_block)
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

    fn gen_i32(&mut self) -> i32 {
        if self.rng.gen_bool(0.5) {
            (self.i32_dist.sample(self.rng) as i64 - i32::MAX as i64) as i32
        } else {
            *[0, 1, -1, i32::MAX, i32::MIN].choose(self.rng).unwrap()
        }
    }

    fn gen_u32(&mut self) -> u32 {
        if self.rng.gen_bool(0.5) {
            (self.u32_dist.sample(self.rng) as i64 - u32::MAX as i64).abs() as u32
        } else {
            *[0, 1, u32::MAX].choose(self.rng).unwrap()
        }
    }

    fn gen_f32(&mut self) -> f32 {
        let k: f64 = self.f32_dist.sample(&mut self.rng);
        let k = if k == 0.0 {
            if self.rng.gen_bool(0.5) {
                1.0
            } else {
                -1.0
            }
        } else {
            k
        };
        let x = k * 1000.0;
        f32::clamp(x.trunc() as f32, -16777216.0, 16777216.0)
    }
}
