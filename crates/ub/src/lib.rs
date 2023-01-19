pub mod cli;
mod ub;

use ast::types::{DataType, ScalarType};
use ast::*;
use std::rc::Rc;
use ub::generate_ub;

// May need more up here basing this off
// of the reconditioner

#[derive(Default)]
pub struct Options {
    blocks: Vec<u32>,
}

pub fn insert_ub(mut ast: Module, flow: Vec<u32>, size: usize) -> Module {
    let ub_struct = StructDecl::new(
        "_WGSLSmithUB",
        vec![
            StructMember::new(
                vec![],
                "min_index".to_string(),
                DataType::Scalar(ScalarType::U32),
            ),
            StructMember::new(
                vec![],
                "max_index".to_string(),
                DataType::Scalar(ScalarType::U32),
            ),
            StructMember::new(
                vec![],
                "write_value".to_string(),
                DataType::Vector(4, ScalarType::U32),
            ),
        ],
    );

    ast.structs.push(ub_struct.clone());

    // largest binding number
    let max_binding = ast
        .vars
        .iter()
        .map(|var| &var.attrs)
        .flatten()
        .map(|attr| match attr {
            GlobalVarAttr::Binding(val) => Some(*val),
            _ => None,
        })
        .flatten()
        .max()
        .unwrap_or(-1);

    ast.vars.push(GlobalVarDecl {
        attrs: vec![
            GlobalVarAttr::Group(0),
            GlobalVarAttr::Binding(max_binding + 1),
        ],
        qualifier: Some(VarQualifier {
            storage_class: StorageClass::Uniform,
            access_mode: None,
        }),
        name: "_wgslsmith_ub".to_string(),
        data_type: DataType::Struct(ub_struct.clone()),
        initializer: None,
    });
    let ub_arr_type = DataType::Array(
        Rc::new(DataType::Vector(4, ScalarType::U32)),
        Some((size / 16) as u32),
    );
    ast.vars.push(GlobalVarDecl {
        attrs: vec![
            GlobalVarAttr::Group(0),
            GlobalVarAttr::Binding(max_binding + 2),
        ],
        qualifier: Some(VarQualifier {
            storage_class: StorageClass::Storage,
            access_mode: Some(AccessMode::ReadWrite),
        }),
        name: "_wgslsmith_ub_arr".to_string(),
        data_type: ub_arr_type.clone(),
        initializer: None,
    });

    let mut inserter = UBInserter::new(
        Options { blocks: flow },
        ub_struct.clone(),
        ub_arr_type.clone(),
    );
    ast.functions = ast
        .functions
        .into_iter()
        .map(|f| inserter.analyze_fn(f))
        .collect::<Vec<_>>();

    // check the reconditioner here

    ast
}

fn get_block_number(flow_stmt: Statement) -> u32 {
    // We parse the statement and grab the number of the block then return it
    // NOTE: Should this take a statement? 
    return 0;
}



// May be more to keep in the state
struct UBInserter {
    block_count: u32,
    blocks: Vec<u32>,
    ub_struct: Rc<StructDecl>,
    arr_type: DataType,
}

impl UBInserter {
    fn new(options: Options, ub_struct: Rc<StructDecl>, arr_type: DataType) -> UBInserter {
        UBInserter {
            block_count: 0,
            blocks: options.blocks,
            ub_struct,
            arr_type,
        }
    }

    fn build_assign(&mut self) -> Option<Statement> {
        println!("C: {}", self.block_count);
        if !self.blocks.contains(&self.block_count) {
            println!("No UB");
            self.block_count += 1;
            return None;
        }
        println!("Adding UB");
        self.block_count += 1;
        Some(generate_ub(self.ub_struct.clone(), self.arr_type.clone()).into())
    }

    fn analyze_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        // Insert the assignment at the beginning of a function
        if !decl.name.starts_with("_wgslsmith_") {
            println!("Fn: {:?}", decl.body[0]);
            //let block = get_block_number(decl.body[0].clone());
            if let Some(insertion) = self.build_assign() {
                decl.body.insert(0, insertion.into());
            }
        }

        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.analyze_stmt(s))
            .collect();

        decl
    }

    fn analyze_else(&mut self, els: Else) -> Else {
        match els {
            Else::If(IfStatement {
                condition,
                body,
                else_,
            }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|s| self.analyze_stmt(s)).collect();
                //let stmts = match &mod_body[0] {
                    //Statement::Compound(e) => Some(e),
                    //_ => None, // Shouldn't get here
                //}.expect("Run flow first (Couldn't find flow statements)");
                //println!("Else If: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                let new_body = if let Some(insertion) = self.build_assign() {
                    vec![Statement::Compound(vec![insertion.into(), mod_body.into()])]
                } else {
                    mod_body
                };
                Else::If(IfStatement {
                    condition,
                    body: new_body,
                    else_: else_.map(|els| Box::new(self.analyze_else(*els))),
                })
            }
            Else::Else(mut stmts) => {
                println!("Else: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                if let Some(insertion) = self.build_assign() {
                    stmts.insert(0, insertion);
                }

                Else::Else(stmts.into_iter().map(|s| self.analyze_stmt(s)).collect())
            }
        }
    }

    fn analyze_stmt(&mut self, stmt: Statement) -> Statement {
        match stmt {
            // The first few matches do nothing since we want to preserve
            // the ast and need an exhaustive match
            Statement::If(IfStatement {
                condition,
                body,
                else_,
            }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|s| self.analyze_stmt(s)).collect();
                //let stmts = match &mod_body[0] {
                    //Statement::Compound(e) => Some(e),
                    //_ => None, // Shouldn't get here
                //}.expect("Run flow first (Couldn't find flow statements)");
                //println!("If: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                let new_body = if let Some(insertion) = self.build_assign() {
                    vec![Statement::Compound(vec![insertion.into(), mod_body.into()])]
                } else {
                    mod_body
                };
                IfStatement::new(condition, new_body)
                    .with_else(else_.map(|els| self.analyze_else(*els)))
                    .into()
            }
            Statement::Loop(LoopStatement { body }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|s| self.analyze_stmt(s)).collect();
                //let stmts = match &mod_body[0] {
                    //Statement::Compound(e) => Some(e),
                    //_ => None, // Shouldn't get here
                //}.expect("Run flow first (Couldn't find flow statements)");
                //println!("Loop: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                let new_body = if let Some(insertion) = self.build_assign() {
                    vec![Statement::Compound(vec![insertion.into(), mod_body.into()])]
                } else {
                    mod_body
                };
                LoopStatement::new(new_body).into()
            }
            Statement::Break => Statement::Break,
            Statement::Switch(SwitchStatement {
                selector,
                cases,
                default,
            }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|SwitchCase { selector, body }| {
                        let mod_body: Vec<Statement> =
                            body.into_iter().map(|it| self.analyze_stmt(it)).collect();
                        //let stmts = match &mod_body[0] {
                            //Statement::Compound(e) => Some(e),
                            //_ => None, // Shouldnt get here check later
                        //}.expect("Run flow first (Couldn't find flow statements)");
                        //println!("switch: {:?}", stmts[0]);
                        //let block = get_block_number(stmts[0].clone());
                        let new_body = if let Some(insertion) = self.build_assign() {
                            vec![Statement::Compound(vec![insertion.into(), mod_body.into()])]
                        } else {
                            mod_body
                        };
                        SwitchCase {
                            selector,
                            body: new_body,
                        }
                    })
                    .collect();
                let mod_default: Vec<Statement> = default
                    .into_iter()
                    .map(|it| self.analyze_stmt(it))
                    .collect();
                //let stmts = match &mod_default[0] {
                    //Statement::Compound(e) => Some(e),
                    //_ => None, // Shouldn't get here
                //}.expect("Run flow first (Couldn't find flow statements)");
                //println!("default: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                let new_default = if let Some(insertion) = self.build_assign() {
                    vec![Statement::Compound(vec![
                        insertion.into(),
                        mod_default.into(),
                    ])]
                } else {
                    mod_default
                };
                SwitchStatement::new(selector, new_cases, new_default).into()
            }
            Statement::ForLoop(ForLoopStatement { header, body }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|it| self.analyze_stmt(it)).collect();
                //let stmts = match &mod_body[0] {
                    //Statement::Compound(e) => Some(e),
                    //_ => None, // Shouldnt get here check later
                //}.expect("Run flow first (Couldn't find flow statements)");
                //println!("ForLoop: {:?}", stmts[0]);
                //let block = get_block_number(stmts[0].clone());
                let new_body = if let Some(insertion) = self.build_assign() {
                    vec![Statement::Compound(vec![insertion.into(), mod_body.into()])]
                } else {
                    mod_body
                };
                ForLoopStatement::new(*header, new_body).into()
            }
            Statement::Continue => Statement::Continue,
            Statement::Fallthrough => Statement::Fallthrough,
            Statement::Compound(stmts) => {
                Statement::Compound(stmts
                    .into_iter()
                    .map(|it| self.analyze_stmt(it))
                    .collect())
            }
            _ => stmt,
        }
    }
}
