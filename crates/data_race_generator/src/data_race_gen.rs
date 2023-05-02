use std::rc::Rc;

use ast::types::{DataType, ScalarType, MemoryViewType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentOp, AssignmentStatement, FnAttr, FnDecl, GlobalVarAttr,
    GlobalVarDecl, Module, ShaderStage, Statement, StorageClass, VarQualifier, ExprNode, Lit, 
    FnInput, FnInputAttr, VarDeclStatement, BinOp, VarExpr, BinOpExpr, Postfix, PostfixExpr
};
use rand::prelude::{SliceRandom, StdRng};
use rand::Rng;

use crate::Options;

enum OpType {
    Literal,
    LocalVar,
    Memory
}

pub struct Generator<'a> {
    rng: &'a mut StdRng,
    options: Rc<Options>,
    vars: Vec<String>,
    lits: Vec<u32>
}

impl<'a> Generator<'a> {
    pub fn new(rng: &'a mut StdRng, options: Rc<Options>) -> Self {
        Generator {
            rng,
            options: options.clone(),
            vars: vec![],
            lits: vec![]
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
                data_type: DataType::Array(Rc::new(ScalarType::U32.into()), Some(1)),
                initializer: None,
            }
        ];

        for _i in 0..self.options.num_lits {
          let val = self.gen_u32();
          self.lits.push(val);
        }

        let mut block: Vec<Statement> = vec![];

        let var_count = self.rng.gen_range(self.options.min_vars..=self.options.max_vars);

        for i in 0..var_count {
            let name = format!("var_{i}");
            self.vars.push(name.clone());
            block.push(self.initialize_var(name));
        }

        let statement_count = self.rng.gen_range(self.options.min_stmts..=self.options.max_stmts);
        for _i in 0..statement_count {
          block.push(self.gen_statement());
        }
        
        let mut local_invocation_id = FnInput::new("local_invocation_id", DataType::Vector(3, ScalarType::U32));
        local_invocation_id.attrs.push(FnInputAttr::Builtin("local_invocation_id".to_string()));
        let mut workgroup_id = FnInput::new("workgroup_id", DataType::Vector(3, ScalarType::U32));
        workgroup_id.attrs.push(FnInputAttr::Builtin("workgroup_id".to_string()));

        let entrypoint = FnDecl {
            attrs: vec![
                FnAttr::Stage(ShaderStage::Compute),
                FnAttr::WorkgroupSize(1),
            ],
            name: "main".to_owned(),
            inputs: vec![workgroup_id, local_invocation_id],
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

    fn initialize_var(&mut self, name: String) -> Statement {
      let ty = ScalarType::U32;
      VarDeclStatement::new(name, Some(ty.into()), None).into()
    }

    fn gen_mem_idx(&mut self) -> ExprNode {
      let local_id = VarExpr::new("local_invocation_id.x").into_node(DataType::from(ScalarType::U32));
      let offset = self.rng.gen_range(0..self.options.locs_per_thread);
      BinOpExpr::new(
        BinOp::Plus,
        local_id,
        ExprNode::from(Lit::U32(offset))).into()
    }

    fn gen_mem_access(&mut self) -> ExprNode {
      let index = Postfix::index(self.gen_mem_idx());
      let arr_expr = VarExpr::new("mem").into_node(
        DataType::Ref(MemoryViewType::new(
    DataType::array(ScalarType::U32, None), StorageClass::Storage)));
      PostfixExpr::new(arr_expr, index).into()
    }

    fn gen_op(&mut self) -> ExprNode {
      let op_types = [OpType::Literal, OpType::LocalVar, OpType::Memory];
       match op_types.choose(self.rng).unwrap() {
        OpType::Literal => ExprNode::from(Lit::U32(self.lits.choose(self.rng).unwrap().clone())),
        OpType::LocalVar => VarExpr::new(self.vars.choose(self.rng).unwrap()).into_node(DataType::from(ScalarType::U32)),
        OpType::Memory => self.gen_mem_access()
      }
    }

    fn gen_expr(&mut self) -> ExprNode {
      BinOpExpr::new(BinOp::Plus, self.gen_op(), self.gen_op()).into()
    }

    fn gen_statement(&mut self) -> Statement {
      let expr = self.gen_expr();
      AssignmentStatement::new(
        AssignmentLhs::array_index(
          "mem", 
          DataType::Ref(MemoryViewType::new(
            DataType::array(ScalarType::U32, None), StorageClass::Storage)),
            self.gen_mem_idx()), 
        AssignmentOp::Simple,
        expr).into()
    }

    fn gen_u32(&mut self) -> u32 {
      *[0, 1, u32::MAX].choose(self.rng).unwrap()
    }
}
