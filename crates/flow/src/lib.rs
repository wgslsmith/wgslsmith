pub mod cli;

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

    let functions = ast
        .functions
        .into_iter()
        .map(|f| flow.analyze_fn(f))
        .collect::<Vec<_>>();

    // check the reconditioner here
    
    ast
}

// May be more to keep in the state
struct Flow {
    block_count: u32,
}

impl Flow {
    fn new(options: Options) -> Flow {
        Flow {
            block_count: 0,
        }
    }

    fn analyze_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        let lhs = AssignmentLhs::array_index("flow.block", 
                                             DataType::Scalar(ScalarType::Bool), 
                                             ExprNode::from(Lit::I32(self.block_count.try_into().unwrap())));
        // Create lhs here which is an assignment with the struct flow, member block
        let op = AssignmentOp::Simple;
        let rhs = Lit::Bool(true); // set the flow to true since we visited
        let assign = AssignmentStatement::new(lhs, op, rhs);

        decl.body
            .insert(0, Statement::Assignment(assign));
        
        // TODO: add in the recursive calls to analyze stmt

        decl
    }
}
