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
    AccessMode, AssignmentLhs, Attr, AttrList, Expr, ExprNode, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, Lit, Module, Postfix, ShaderStage, Statement, StorageClass, StructDecl,
    StructMember, VarQualifier,
};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::generator::scope::Scope;
use crate::Options;

use self::cx::Context;
use self::stmt::BlockContext;

pub struct Generator {
    rng: StdRng,
    options: Rc<Options>,
}

impl Generator {
    pub fn new(rng: StdRng, options: Rc<Options>) -> Self {
        Generator { rng, options }
    }

    #[tracing::instrument(skip(self))]
    pub fn gen_module(&mut self) -> Module {
        let mut cx = Context::new(self.options.clone());

        let struct_count = self
            .rng
            .gen_range(self.options.min_structs..=self.options.max_structs);

        for i in 1..=struct_count {
            let name = format!("Struct_{}", i);
            let decl = structs::gen_struct_decl(&mut self.rng, &mut cx, &self.options, name);
            cx.types.get_mut().insert(decl);
        }

        let entrypoint = self.gen_entrypoint_function(&Scope::empty(), &mut cx);
        let Context { types, fns } = cx;

        let buffer_type_decl = StructDecl::new(
            "Buffer",
            vec![StructMember::new(
                "data",
                DataType::Array(Rc::new(DataType::Scalar(ScalarType::U32))),
            )],
        );

        Module {
            structs: {
                let mut structs = types.into_inner().into_structs();
                structs.push(buffer_type_decl.clone());
                structs
            },
            vars: vec![GlobalVarDecl {
                attrs: AttrList(vec![
                    self.gen_attr(GlobalVarAttr::Group(0)),
                    self.gen_attr(GlobalVarAttr::Binding(0)),
                ]),
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Storage,
                    access_mode: Some(AccessMode::ReadWrite),
                }),
                name: "output".to_owned(),
                data_type: DataType::Struct(buffer_type_decl),
                initializer: None,
            }],
            functions: fns.into_inner().into_fns(),
            entrypoint,
        }
    }

    fn gen_attr<T>(&mut self, attr: T) -> Attr<T> {
        Attr {
            attr,
            style: *self.options.attribute_style.choose(&mut self.rng).unwrap(),
        }
    }

    #[tracing::instrument(skip(self, scope, cx))]
    fn gen_entrypoint_function(&mut self, scope: &Scope, cx: &mut Context) -> FnDecl {
        let stmt_count = self.rng.gen_range(5..10);
        let (scope, mut block) = stmt::gen_block(
            &mut self.rng,
            cx,
            scope,
            &BlockContext::new(None),
            &self.options,
            stmt_count,
        );

        block.push(Statement::Assignment(
            AssignmentLhs::Simple(
                "output".to_owned(),
                vec![
                    Postfix::Member("data".to_owned()),
                    Postfix::ArrayIndex(Box::new(ExprNode {
                        data_type: DataType::Scalar(ScalarType::U32),
                        expr: Expr::Lit(Lit::UInt(0)),
                    })),
                ],
            ),
            expr::gen_expr(
                &mut self.rng,
                cx,
                &scope,
                &self.options,
                &DataType::Scalar(ScalarType::U32),
            ),
        ));

        FnDecl {
            attrs: AttrList(vec![
                self.gen_attr(FnAttr::Stage(ShaderStage::Compute)),
                self.gen_attr(FnAttr::WorkgroupSize(1)),
            ]),
            name: "main".to_owned(),
            inputs: vec![],
            output: None,
            body: block,
        }
    }
}
