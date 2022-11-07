pub mod cli;

use ast::types::{DataType, MemoryViewType, ScalarType};
use ast::*;

// May need more up here basing this off
// of the reconditioner

#[derive(Default)]
pub struct Options; // No opts yet

pub fn flow(ast: Module) -> Module {
    flow_with(ast, Options::default())
}

pub fn flow_with(mut ast: Module, options: Options) -> Module {
    let mut flow = Flow::new(options);
    
    ast.functions = ast
        .functions
        .into_iter()
        .map(|f| flow.analyze_fn(f))
        .collect::<Vec<_>>();

    let flow_struct = StructDecl::new(
        "_WGSLSmithFlow",
        vec![StructMember::new(
            vec![],
            "block".to_string(),
            DataType::array(DataType::Scalar(ScalarType::AU32), Some(flow.block_count)),
        )],
    );
    ast.structs.push(flow_struct.clone());

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
            storage_class: StorageClass::Storage,
            access_mode: Some(AccessMode::ReadWrite),
        }),
        name: "_wgslsmith_flow".to_string(),
        data_type: DataType::Struct(flow_struct),
        initializer: None,
    });
    // check the reconditioner here

    ast
}

// May be more to keep in the state
struct Flow {
    block_count: u32,
}

impl Flow {
    fn new(_options: Options) -> Flow {
        Flow { block_count: 0 }
    }

    fn build_assign(&mut self) -> FnCallStatement {
        // We will use an i32 array so we can see overflow
        // TODO: Check if uniform is correct
        //let lhs = AssignmentLhs::array_index(
            //"_wgslsmith_flow.block",
            //DataType::Ref(MemoryViewType::new(
                //DataType::array(ScalarType::U32, None),
                //StorageClass::Uniform,
            //)),
            //ExprNode::from(Lit::U32(self.block_count.try_into().unwrap())),
        //);
        // Create lhs here which is an assignment with the struct flow, member block
        //let op = AssignmentOp::Plus;
        //let rhs = ExprNode::from(Lit::U32(1)); // set the flow to true since we visited
        //let assign = AssignmentStatement::new(lhs, op, rhs);


        // Build args and then build the statement
        let index = Postfix::index(ExprNode::from(Lit::U32(self.block_count.try_into().unwrap())));
        let arr_expr = VarExpr::new("_wgslsmith_flow.block").into_node(
                            DataType::Ref(MemoryViewType::new(
                                DataType::array(ScalarType::AU32, None),
                                StorageClass::Uniform,
                            )));
        let indexed_arr = PostfixExpr::new(arr_expr, index);
        let first_arg = UnOpExpr::new(UnOp::AddressOf, indexed_arr);
        let second_arg = ExprNode::from(Lit::U32(1));
        let args: Vec<ExprNode> = vec![first_arg.into(), second_arg];

        let assign = FnCallStatement::new(String::from("atomicAdd"), args);
        
        self.block_count += 1;

        assign
    }

    fn analyze_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        // Insert the assignment at the beginning of a function
        if !decl.name.starts_with("_wgslsmith_") {
            decl.body
                .insert(0, Statement::FnCall(self.build_assign()));
        }

        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.analyze_stmt(s))
            .collect();

        // TODO: add in the recursive calls to analyze stmt

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
                let new_body = vec![Statement::Compound(vec![
                    self.build_assign().into(),
                    mod_body.into(),
                ])];
                Else::If(IfStatement {
                    condition,
                    body: new_body,
                    else_: else_.map(|els| Box::new(self.analyze_else(*els))),
                })
            }
            Else::Else(mut stmts) => {
                stmts.insert(0, Statement::FnCall(self.build_assign()));

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
                let new_body = vec![Statement::Compound(vec![
                    self.build_assign().into(),
                    mod_body.into(),
                ])];
                IfStatement::new(condition, new_body)
                    .with_else(else_.map(|els| self.analyze_else(*els)))
                    .into()
            }
            Statement::Loop(LoopStatement { body }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|s| self.analyze_stmt(s)).collect();
                let new_body = vec![Statement::Compound(vec![
                    self.build_assign().into(),
                    mod_body.into(),
                ])];
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
                        let new_body = vec![Statement::Compound(vec![
                            self.build_assign().into(),
                            mod_body.into(),
                        ])];
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
                let new_default = vec![Statement::Compound(vec![
                    self.build_assign().into(),
                    mod_default.into(),
                ])];
                SwitchStatement::new(selector, new_cases, new_default).into()
            }
            Statement::ForLoop(ForLoopStatement { header, body }) => {
                let mod_body: Vec<Statement> =
                    body.into_iter().map(|it| self.analyze_stmt(it)).collect();
                let new_body = vec![Statement::Compound(vec![
                    self.build_assign().into(),
                    mod_body.into(),
                ])];
                ForLoopStatement::new(*header, new_body).into()
            }
            Statement::Continue => Statement::Continue,
            Statement::Fallthrough => Statement::Fallthrough,
            _ => stmt,
        }
    }
}
