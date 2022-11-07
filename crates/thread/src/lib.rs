pub mod cli;

use ast::types::{DataType, MemoryViewType};
use ast::*;

// TODO: Add opts for the thread size for now just pass an int

pub fn thread(mut ast: Module, workg_count: u32, dispatch_size: u32) -> Module {
    let mut dtype = None;
    for gl_var in &ast.vars {
        if gl_var.name == "s_output" { // Target name of output (this is bad but quick to write)
            dtype = Some(gl_var.data_type.clone()); // Copy it for later
            break;
        }   
    }

    ast.vars = ast
        .vars
        .into_iter()
        .map(|v| {
            if v.name == "s_output" {
                let mut new_var = v.clone();
                new_var.data_type = DataType::array(dtype.as_ref().expect("No type for output").clone(), workg_count);
                new_var
            } else {
                v
            }
        })
        .collect::<Vec<_>>();
    // We wont follow the same pattern as the other reconditioning crates since we need
    // thread_count and block count
    let mut thread = Thread::new(workg_count, dispatch_size, dtype.expect("No type for output"));
    
    // Analyze the functions to change the ending write and the function decorator of main
    ast.functions = ast
        .functions
        .into_iter()
        .map(|f| thread.analyze_fn(f))
        .collect::<Vec<_>>();

    // Modify the output buffer to be an array, perhaps simply reconstruct it?

    ast
}

struct Thread {
    workg_count: u32,
    dispatch_size: u32,
    out_type: DataType,
}

impl Thread {
    fn new(workg_count: u32, dispatch_size: u32, t: DataType) -> Thread {
        Thread {
            workg_count: workg_count,
            dispatch_size: dispatch_size,
            out_type: t,
        }
    }

    fn analyze_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        if decl.name == "main" {
            // modify if function is main

            // add the builtin threadId vector
            let mut input = FnInput::new("LocalInvocationID", DataType::Vector(3, ScalarType::U32));
            input.attrs.push(FnInputAttr::Builtin("local_invocation_id".to_string()));
            
            decl.inputs.push(input);

            decl.attrs = decl
                .attrs.into_iter()
                .map(|atr| {
                    match atr {
                        // Change workgroup size
                        FnAttr::WorkgroupSize(_) => FnAttr::WorkgroupSize(self.workg_count),
                        // Leave any other attr alone
                        _ => atr,
                    }
                })
                .collect::<Vec<_>>();

            // Lets make an assumption here that our file will look like a wgslsmith file
            let assign = decl.body.last().expect("Requires program generated by wgslsmith");
            let mut out_name = None; // Declare here and assign in if lets

            if let Statement::Assignment(stmt) = assign { // Grabbed assignstmt
                let lhs = &stmt.lhs;
                let op = AssignmentOp::Simple; // Grab = to construct
                let rhs = match stmt.rhs.expr.clone() {
                    Expr::Lit(e) => Some(ExprNode::from(e)),
                    Expr::TypeCons(e) => Some(ExprNode::from(e)),
                    Expr::Postfix(e) => Some(ExprNode::from(e)),
                    Expr::UnOp(e) => Some(ExprNode::from(e)),
                    Expr::BinOp(e) => Some(ExprNode::from(e)),
                    Expr::FnCall(e) => Some(e.into_node(self.out_type.clone())),
                    _ => None,
                }.expect("Invalid assignment (not supported)");
                
                if let AssignmentLhs::Expr(expr_node) = lhs { // Grabbed expr
                    let expr = &expr_node.expr;

                    if let LhsExpr::Ident(name) = expr { // grabbed var name
                        out_name = Some(name.clone()); // Clone name to not lose as we stop borrows
                    }
                }
                // Set lhs to new indexed Lhs we need to grab the thread id for indexing
                let new_lhs = AssignmentLhs::array_index(
                    out_name.expect("Missing out name"),
                    // We need to grab the data type in the start
                    DataType::Ref(MemoryViewType::new(
                        DataType::array(self.out_type.clone(), self.workg_count),
                        StorageClass::Storage,
                    )),
                    VarExpr::new("LocalInvocationID.x").into_node(DataType::from(ScalarType::U32)),
                );
                decl.body.pop(); // remove old assign
                decl.body.push(Statement::Assignment(AssignmentStatement::new(new_lhs, op, rhs)));
            }
        }

        decl
    }
}