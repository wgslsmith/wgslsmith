use std::rc::Rc;

use ast::types::{DataType, ScalarType, MemoryViewType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentOp, AssignmentStatement, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, LetDeclStatement, Module, Postfix, PostfixExpr, ShaderStage, Statement,
    StorageClass, VarExpr, VarQualifier, ExprNode, Lit
};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::Options;

pub struct Generator<'a> {
    rng: &'a mut StdRng,
    options: Rc<Options>,
}

impl<'a> Generator<'a> {
    pub fn new(rng: &'a mut StdRng, options: Rc<Options>) -> Self {
        Generator {
            rng,
            options: options.clone()
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn gen_module(&mut self) -> Module {
        let global_vars = vec![
            GlobalVarDecl {
                attrs: vec![GlobalVarAttr::Group(0), GlobalVarAttr::Binding(0)],
                qualifier: Some(VarQualifier {
                    storage_class: StorageClass::Storage,
                    access_mode: Some(AccessMode::ReadWrite),
                }),
                name: "mem".to_owned(),
                data_type: DataType::Array(Rc::new(DataType::Scalar(ScalarType::U32)), Some(1)),
                initializer: None,
            }
        ];

        let mut block: Vec<Statement> = vec![];
        block.push(                   
          AssignmentStatement::new(
            AssignmentLhs::array_index(
              "mem", 
              DataType::Ref(MemoryViewType::new(
                  DataType::Array(Rc::new(DataType::Scalar(ScalarType::U32)), None), 
                  StorageClass::Storage)),
              Lit::U32(0).into()), 
            AssignmentOp::Simple, 
            ExprNode { 
              data_type: ScalarType::U32.into(), 
              expr: Lit::U32(1).into()
            }).into());
        let entrypoint = FnDecl {
            attrs: vec![
                FnAttr::Stage(ShaderStage::Compute),
                FnAttr::WorkgroupSize(1),
            ],
            name: "main".to_owned(),
            inputs: vec![],
            output: None,
            body: block,
        };

        let functions = vec![entrypoint];

        Module {
            structs: vec![],
            consts: vec![],
            vars: global_vars,
            functions,
        }
    }
}
