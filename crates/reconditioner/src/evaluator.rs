use ast::*;

/* Concretized node struct contains a concretized node and 
   the value from evaluating that node. The value is None
   if the node cannot be evaluated at shader creation time
   (i.e. the node is not a const expression, for example if
   it contains a runtime variable).
*/

macro_rules! binop {
    ($data_type:expr, $op:expr, $l:expr, $r:expr) => {
        
        match $op {
            BinOp::Plus => $l + $r,
            BinOp::Minus => $l - $r,
            BinOp::Times => $l * $r,
            BinOp::Divide => $l / $r,
            BinOp::Mod => $l % $r,
            _ => todo!(),
         }
    }
}

macro_rules! binop_shift {
    ($data_type:expr, $op:expr, $l:expr, $r:expr) => {
        
        match $op {
            BinOp::LShift => $l << $r,
            BinOp::RShift => $l >> $r,
            _ => todo!(),
         }
    }
}
 

#[derive(Clone)]
pub enum Value {
    Lit(Lit),
    Vector(Vec<Value>)
}

#[derive(Clone)]
pub struct ConNode {
    node : ExprNode,
    value : Option<Value>,
}


// from ExprNode to ConNode (for passing into concretize fns)
impl From<ExprNode> for ConNode {
    fn from(node : ExprNode) -> Self {
        ConNode {
            node : node,
            value : None,
        }
     }
}

// from ConNode to ExprNode (for extracting expr to rebuild program, when we don't care about values)
impl From<ConNode> for ExprNode {
    fn from(con : ConNode) -> Self {
        ExprNode {
            data_type : con.node.data_type,
            expr : con.node.expr,
        }
    }
}


#[derive(Default)]
pub struct Options {
    pub placeholder: bool,
}

pub fn concretize(ast: Module) -> Module {
    concretize_with(ast, Options::default())
}

pub fn concretize_with(mut ast: Module, options: Options) -> Module {
    let mut evaluator = Evaluator::new(options);

    // Concretize the functions
    let functions = ast
        .functions
        .into_iter()
        .map(|f| evaluator.concretize_fn(f))
        .collect::<Vec<_>>();

    // Reassign the concretized functions to ast
    ast.functions = functions;

    ast

}

struct Evaluator {

    // keep track of which internal variables are concretizable
    // as we traverse the AST
    placeholder: bool,

}

impl Evaluator {

    fn new(options: Options) -> Evaluator {
        Evaluator {
            placeholder: options.placeholder,
        }
    }

    fn concretize_fn(&self, mut decl: FnDecl) -> FnDecl {

        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.concretize_stmt(s))
            .collect();
        
        decl
    }

   fn concretize_stmt(&self, stmt: Statement) -> Statement {

        //TODO: if stmt contains var, return (since not concretizable)

        match stmt {
            Statement::LetDecl(LetDeclStatement {ident, initializer}) => {
                LetDeclStatement::new(ident, self.concretize_expr(initializer)).into()
            }
            Statement::VarDecl(VarDeclStatement {
                ident,
                data_type,
                initializer,
            }) => VarDeclStatement::new(
                ident,
                data_type,
                initializer.map(|e| self.concretize_expr(e).into()),
            ).into(),
            Statement::Assignment(AssignmentStatement {lhs, op, rhs}) => {
                AssignmentStatement::new(
                    lhs,
                    op,
                    self.concretize_expr(rhs.into()),
                ).into()
            },
            Statement::Compound(s) => {
                Statement::Compound(s.into_iter().map(|s| self.concretize_stmt(s)).collect())
            },
            Statement::If(IfStatement {
                condition,
                body,
                else_,
            }) => IfStatement::new(
                    self.concretize_expr(condition.into()),
                    body.into_iter().map(|s| self.concretize_stmt(s)).collect(),
                    )
                .with_else(else_.map(|els| *els))
                .into(),
            Statement::Return(ReturnStatement {value}) => ReturnStatement {
                value: value.map(|e| self.concretize_expr(e).into()),
            }.into(),
            Statement::Switch(SwitchStatement {
                selector,
                cases,
                default,
            }) => SwitchStatement::new(
                self.concretize_expr(selector).into(),
                cases
                    .into_iter()
                    .map(|c| self.concretize_switch_case(c))
                    .collect(),
                default
                    .into_iter()
                    .map(|s| self.concretize_stmt(s))
                    .collect(),
                ).into(),
            Statement::FnCall(FnCallStatement {ident, args}) => FnCallStatement::new(
                ident,
                args
                    .into_iter()
                    .map(|e| self.concretize_expr(e).into())
                    .collect()
                ).into(),
            Statement::Loop(LoopStatement {body}) => LoopStatement::new(
                body.into_iter().map(|s| self.concretize_stmt(s)).collect()).into(),
            Statement::ForLoop(ForLoopStatement {header, body}) => ForLoopStatement::new(
                ForLoopHeader {
                    init : header.init.map(|init| self.concretize_for_init(init)),
                    condition: header.condition.map(|e| self.concretize_expr(e).into()),
                    update : header.update.map(|update| self.concretize_for_update(update)),
                    },
                body.into_iter().map(|s| self.concretize_stmt(s)).collect(),
                ).into(),
            Statement::Break => Statement::Break,
            Statement::Continue => Statement::Continue,
            Statement::Fallthrough => Statement::Fallthrough,
        }
   }

   fn concretize_for_init(&self, init : ForLoopInit) -> ForLoopInit {
       match init {
           ForLoopInit::VarDecl(VarDeclStatement {
               ident,
               data_type,
               initializer,
           }) => ForLoopInit::VarDecl(VarDeclStatement::new(
               ident,
               data_type,
               initializer.map(|e| self.concretize_expr(e).into()),
               )),
       }
   }

    fn concretize_for_update(&self, update : ForLoopUpdate) -> ForLoopUpdate {
       match update {
           ForLoopUpdate::Assignment(AssignmentStatement {
               lhs,
               op,
               rhs,
           }) => { ForLoopUpdate::Assignment(AssignmentStatement::new(
               self.concretize_assignment_lhs(lhs),
               op,
               self.concretize_expr(rhs)
            ))
           }
       }
    }

    fn concretize_assignment_lhs(&self, lhs : AssignmentLhs) -> AssignmentLhs {
        match lhs {
            AssignmentLhs::Phony => AssignmentLhs::Phony,
            AssignmentLhs::Expr(expr) => AssignmentLhs::Expr(self.concretize_lhs_expr(expr)),
        }.into()
    }

    fn concretize_lhs_expr(&self, node : LhsExprNode) -> LhsExprNode {
        let expr = match node.expr {
            LhsExpr::Ident(ident) => LhsExpr::Ident(ident),
            LhsExpr::Postfix(expr, postfix) => LhsExpr::Postfix(
                self.concretize_lhs_expr(*expr).into(),
                match postfix {
                    Postfix::Index(index) => Postfix::Index(Box::new(self.concretize_expr(*index).into())),
                    Postfix::Member(string) => Postfix::Member(string),
                }),
            LhsExpr::Deref(_) => todo!(),
            LhsExpr::AddressOf(_) => todo!(),
        };

        LhsExprNode{
            data_type: node.data_type,
            expr : expr,
            }
   }
   
    fn concretize_switch_case(&self, case: SwitchCase) -> SwitchCase {
       
       let concretized_selector = self.concretize_expr(case.selector);

       let concretized_body = case
           .body
           .into_iter()
           .map(|s| self.concretize_stmt(s))
           .collect();

       SwitchCase {
           selector : concretized_selector.into(),
           body : concretized_body,
       }
   }

    fn concretize_expr(&self, node: ExprNode) -> ConNode {

        //TODO: if expr contains var, return (since not concretizable)

        match node.expr {
            Expr::Lit(lit) => self.test_lit(lit), // placeholder
            
            Expr::TypeCons(expr) => {
                return self.concretize_typecons(node.data_type, expr);
            }
                        Expr::UnOp(expr) => {

                let con_inner = self.concretize_expr(*expr.inner);

                return self.concretize_unop(node.data_type, expr.op, con_inner);
            }
            Expr::BinOp(expr) => {
                let left = self.concretize_expr(*expr.left);
                let right = self.concretize_expr(*expr.right);

                return self.concretize_bin_op(node.data_type, expr.op, left, right);
            }
            Expr::FnCall(expr) => ConNode { 
                node : ExprNode { data_type : node.data_type, expr : expr.into()}, 
                value : None}, //TODO
            Expr::Postfix(expr) =>  ConNode { 
                node : ExprNode { data_type : node.data_type, expr : expr.into()}, 
                value : None}, //TODO
            Expr::Var(expr) => ConNode {
                node : ExprNode { data_type : node.data_type, expr : expr.into()},
                value : None}, //TODO
        }   

        
        
    }

    fn concretize_typecons(
        &self, 
        data_type : DataType, 
        expr : TypeConsExpr
        ) -> ConNode {

        let concrete_args : Vec<ConNode> = expr.args
            .into_iter()
            .map(|e| self.concretize_expr(e))
            .collect();

        let none_values = concrete_args
            .iter()
            .filter(|c| c.value.is_none())
            .count();

        let mut new_node : Vec<ExprNode> = Vec::new();
        let mut new_val : Vec<Value> = Vec::new();

        for ConNode {node, value} in concrete_args {
            new_node.push(node);
            if none_values == 0 {
                new_val.push(value.unwrap());
            };
        }

        if none_values > 0 {
            return ConNode {
                node : TypeConsExpr::new(
                        data_type,
                        new_node,
                ).into(),
                value : None,
            };
        }

        else {
            return ConNode {
                node : TypeConsExpr::new(
                       data_type,
                       new_node,
                ).into(),
                value : Some(Value::Vector(new_val)),
            };
        }
        
    }



    fn test_lit(&self, lit : Lit) -> ConNode {
       
        //TODO: placeholder to test operation of concretization 
        let value = match lit {
            Lit::I32(_) => Lit::I32(1),
            Lit::U32(_) => Lit::U32(1),
            Lit::F32(_) => Lit::F32(1.0),
            Lit::Bool(b) => Lit::Bool(b),
        };

        ConNode {
            node : ExprNode {
                data_type : lit.data_type(),
                expr : Expr::Lit(value),
            },
            value : Some(Value::Lit(value)),
        }
    }

    fn concretize_bin_op(
        &self, 
        data_type : DataType, 
        op : BinOp, 
        left : ConNode, 
        right : ConNode
    ) -> ConNode {
       
        // if either left or right is not a const-expression, then 
        // this node is not a const-expression
        if left.value.is_none() || right.value.is_none() {
            return ConNode {
                node : ExprNode {
                    data_type : data_type,
                    expr: Expr::BinOp(BinOpExpr::new(op, left, right))
                },
                value : None,
            }
        }
        
        let value = self.evaluate_bin_op(&data_type, &op, left.clone(), right.clone());
   
        if self.within_bounds(value.clone()) {
            return ConNode {
                node : ExprNode {
                    data_type : data_type,
                    expr : Expr::BinOp(BinOpExpr::new(op, left, right))
                },
                value : value,
            };
        }
        else {
            return self.default_concrete_binop(data_type, op);
        }

    }
            

    fn default_concrete_binop(
        &self,
        data_type : DataType,
        op : BinOp,
    ) -> ConNode {

        let (left, right) = match data_type {
            DataType::Scalar(ScalarType::I32) => (Lit::I32(1), Lit::I32(1)),
            DataType::Scalar(ScalarType::U32) => (Lit::U32(1), Lit::U32(1)),
            DataType::Scalar(ScalarType::F32) => (Lit::F32(1.0), Lit::F32(1.0)),
            _ => todo!()

        };

        let concrete_value = self.evaluate_bin_op(
            &data_type, 
            &op,
            ConNode{node : left.into(), value : Some(Value::Lit(left))},
            ConNode{node : right.into(), value : Some(Value::Lit(right))},
        );

        // set up default nodes with appropriate type
        ConNode {
            node : ExprNode {
                data_type: data_type,
                expr: Expr::BinOp(BinOpExpr::new(op, left, right))
            },
            value : concrete_value,
        }

    }

    fn evaluate_bin_op(
        &self,
        data_type : &DataType,
        op : &BinOp,
        l : ConNode,
        r : ConNode
    ) -> Option<Value> {


        match (l.value.unwrap(), r.value.unwrap()) {
           
            (Value::Vector(lv), Value::Vector(rv)) => {
                let result = self.eval_bin_op_vector(data_type, op, lv, rv);

                return None::<Value>;
            }
/*
            // type cons expressions
            (Value::TypeCons(TypeConsExpr {l_dt, l_args}), Value::TypeCons(TypeConsExpr {r_dt, r_args})) => {
                let result = eval_bin_op_typecons(data_type, l_args, r_args);

                return None::<Value;
            }
*/
            (Value::Lit(Lit::I32(lv)), Value::Lit(Lit::I32(rv))) => {
                
                let result = match op {
                    BinOp::LShift | BinOp::RShift => binop_shift!(data_type, op, lv, rv),
                    _ => binop!(data_type, op, lv, rv),
                };

                return Some(Value::Lit(Lit::I32(result)));
                
            },

            (Value::Lit(Lit::U32(lv)), Value::Lit(Lit::U32(rv))) => {
                
                let result = match op {
                    BinOp::LShift | BinOp::RShift => binop_shift!(data_type, op, lv, rv),
                    _ => binop!(data_type, op, lv, rv),
                };

                return Some(Value::Lit(Lit::U32(result)));
                
            },
            
            (Value::Lit(Lit::F32(lv)), Value::Lit(Lit::F32(rv))) => {
                
                let result = binop!(data_type, op, lv, rv);

                return Some(Value::Lit(Lit::F32(result)));
            },
            _ => None::<Value>,
        }
        
    }

    fn eval_bin_op_vector(
        &self,
        data_type : &DataType,
        op : &BinOp,
        l : Vec<Value>,
        r : Vec<Value>
    ) ->  Option<Value> {
        return Some(Value::Vector(l));
    }

    fn concretize_unop(
        &self,
        data_type : DataType,
        op : UnOp,
        inner : ConNode
    ) -> ConNode {

        // evaluate unop
        let value = None;
        /*
        let value = match op {
            UnOp::Neg => todo!(),
            UnOp::Not => todo!(),
            UnOp::BitNot => todo!(),
            UnOp::AddressOf => todo!(),
            UnOp::Deref => todo!(),
        };
        */
        // check if value is out of bounds
        // and replace value and  inner node if so
        let concrete_inner = inner.node; //TODO:implement check
        let concrete_value = value;

        // create new unop with updated node
        ConNode {
            node : ExprNode {
                data_type : data_type,
                expr : Expr::UnOp(UnOpExpr::new(
                        op,
                        concrete_inner)),
            },
            value : concrete_value,
        }

   }

    // TODO: implement
    fn within_bounds(&self, value : Option<Value>) -> bool {
       
        match value.unwrap() {
            Value::Lit(Lit::Bool(_)) => return true,
            _ => return false,
        }
   }

}
