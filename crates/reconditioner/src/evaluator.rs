use ast::*;
use crate::eval_value::Value;
use crate::eval_builtin::*;

/* Concretized node struct contains a concretized node and 
   the value from evaluating that node. The value is None
   if the node cannot be evaluated at shader creation time
   (i.e. the node is not a const expression, for example if
   it contains a runtime variable).
*/
macro_rules! binop_int_arith {
    ($op:expr, $l:expr, $r:expr) => {
        match $op {
            BinOp::Plus => ($l).checked_add($r),
            BinOp::Minus =>($l).checked_sub($r),
            BinOp::Times => ($l).checked_mul($r),
            BinOp::Divide => ($l).checked_div($r),
            BinOp::Mod => ($l).checked_rem($r),
            _ => None, 
        }
    }
}

/* WGSL shift evaluation rules involve two checks:
   e1 << e2 : T
   (1) bit width of e1 must be >= value of e2
   (2) result must not overflow
   analagous for >>
   Macro performs first check for bit width
*/
macro_rules! binop_int_shift {
     ($op:expr, $l:expr, $r:expr) => {
        
         // rust checked_shl and checked_shr do not
         // include overflow checks
        match ($l, $r) {
            (Lit::I32(l), Lit::U32(r)) => {
                match $op {
                    BinOp::LShift => Value::from_i32(l.checked_shl(r)),
                    BinOp::RShift => Value::from_i32(l.checked_shr(r)),
                    _ => None,
            }},
            (Lit::U32(l), Lit::U32(r)) => {
                match $op {
                    BinOp::LShift => Value::from_u32(l.checked_shl(r)),
                    BinOp::RShift => Value::from_u32(l.checked_shr(r)),
                    _ => None,
            }},
            _ => None,
        }
     }
}

fn binop_float(op : &BinOp, l : f32, r : f32) -> Option<f32> {
        
        let result = match op {
            BinOp::Plus => l + r,
            BinOp::Minus => l - r,
            BinOp::Times => l * r,
            BinOp::Divide => l / r,
            BinOp::Mod => l % r,
            _ => todo!(), 
        };

        return in_float_range(result); 

        // TODO: add float division and mod check
}

fn in_float_range(f : f32) -> Option<f32> {
    if f.abs() <= 0.1_f32 || f.abs() >= (16777216_f32) {
        return None;
    }
    else {return Some(f);}
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
    pub _placeholder: bool,
}

pub fn concretize(ast: Module) -> Module {
    concretize_with(ast, Options::default())
}

pub fn concretize_with(mut ast: Module, options: Options) -> Module {

    let evaluator = Evaluator::new(options);

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
    _placeholder: bool,

}

impl Evaluator {

    fn new(options: Options) -> Evaluator {
        Evaluator {
            _placeholder: options._placeholder,
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
            Expr::Lit(lit) => {return self.concretize_lit(lit);},
            Expr::TypeCons(expr) => {
                return self.concretize_typecons(node.data_type, expr);
            },
            Expr::UnOp(expr) => {

                let con_inner = self.concretize_expr(*expr.inner);

                return self.concretize_unop(node.data_type, expr.op, con_inner);
            },
            Expr::BinOp(expr) => {
                let left = self.concretize_expr(*expr.left);
                let right = self.concretize_expr(*expr.right);

                return self.concretize_bin_op(node.data_type, expr.op, left, right);
            },
            Expr::FnCall(expr) => { 
                let concrete_args = expr.args
                    .into_iter()
                    .map(|e| self.concretize_expr(e))
                    .collect();

                return self.concretize_fncall(node.data_type, expr.ident, concrete_args);
            },
            Expr::Postfix(expr) =>  {
                let concrete_inner = self.concretize_expr(*expr.inner);

                let concrete_postfix = match expr.postfix {
                    Postfix::Index(index) => Postfix::Index(Box::new(self.concretize_expr(*index).into())),
                    Postfix::Member(string) => Postfix::Member(string),
                };

                return ConNode {
                    node : PostfixExpr::new(concrete_inner, concrete_postfix).into(),
                    value : None
                };
            }, 
            Expr::Var(expr) => { 
                ConNode {
                node : ExprNode { data_type : node.data_type, expr : expr.into()},
                value : None //TODO
            }},
        }   

        
        
    }

    fn concretize_fncall(&self,
                     data_type : DataType,
                     ident : String,
                     args : Vec<ConNode> 
                     ) -> ConNode {

        // nodes : Vec<ExprNode>, vals : Option<Value>
        let (nodes, vals) = self.decompose_vec_con(args);

        // return node with none if any vals are none
        if self.contains_none(&vals) {
            return ConNode {
                node : FnCallExpr::new(ident, nodes).into_node(data_type),
                value : None,
            };
        }

        let function = Builtin::convert(ident.clone());

        match function {
            // Evaluate function result and determine whether the node must
            // be replaced with a default
            Some(f) => {
                let evaluated_val = evaluate_builtin(&f, vals); 
                
                match evaluated_val {
                    Some(_) => ConNode {
                        node: FnCallExpr::new(ident, nodes).into_node(data_type),
                        value : evaluated_val,
                        },
                    None => self.default_node(data_type),
                }
            },
            // If the function ident is not implemented in eval_builtin, then
            // simply return the same node with a None value
            None => {
                ConNode {
                    node : FnCallExpr::new(ident, nodes).into_node(data_type),
                    value : None,
                }
            },

        }
    }

    fn contains_none(&self, vals : &Vec<Option<Value>>) -> bool {

        vals.iter().any(|v| v.is_none())
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

        let (new_node, new_val) = self.decompose_vec_con(concrete_args);

        let new_val = if self.contains_none(&new_val) {None}
            else {Some(Value::Vector(new_val.into_iter().map(|v| v.unwrap()).collect()))};

        return ConNode {
            node : TypeConsExpr::new(
                    data_type,
                    new_node,
            ).into(),
            value : new_val,
        }
       
    }

    // function decomposes Vec<ConNode> to a tuple
    // of (Vec<ExprNode>, Vec<Value>)
    fn decompose_vec_con(
        &self,
        vec : Vec<ConNode>
    ) -> (Vec<ExprNode>, Vec<Option<Value>>) {

        let mut new_node : Vec<ExprNode> = Vec::new();
        let mut new_val : Vec<Option<Value>> = Vec::new();

        for ConNode {node, value} in vec {
            new_node.push(node);
            new_val.push(value);
        }

        (new_node, new_val)
    }




    fn concretize_lit(&self, lit : Lit) -> ConNode {
        ConNode {
            node : ExprNode {
                data_type : lit.data_type(),
                expr : Expr::Lit(lit),
            },
            value : Some(Value::Lit(lit)),
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
        
        let value : Option<Value>= self.evaluate_bin_op(
            &data_type, 
            &op, 
            left.value.clone().unwrap(), 
            right.value.clone().unwrap()
            );
   
        if value.is_none() {
            return self.default_node(data_type);
        }
        else {
            return ConNode {
                node : ExprNode {
                    data_type : data_type,
                    expr : Expr::BinOp(BinOpExpr::new(op, left, right))
                },
                value : value,
            };
        }

    }
    
    fn default_node(
        &self,
        data_type : DataType
    ) -> ConNode {

        match data_type {
            DataType::Scalar(ty) => {
                match ty {
                    ScalarType::U32 => ConNode {
                        node : Lit::U32(1_u32).into(),
                        value : Value::from_u32(Some(1_u32)),
                        },
                    ScalarType::I32 =>ConNode {
                        node : Lit::I32(1_i32).into(),
                        value : Value::from_i32(Some(1_i32)),
                        },
                    ScalarType::F32 =>ConNode {
                        node : Lit::F32(1_f32).into(),
                        value : Value::from_f32(Some(1_f32)),
                        },
                    ScalarType::Bool =>ConNode {
                        node : Lit::Bool(true).into(),
                        value : Value::from_bool(Some(true)),
                        },
                }
            },
            DataType::Vector(size, ty) => {
                match ty {
                    ScalarType::U32 => ConNode {
                        node : TypeConsExpr::new(
                                   data_type,
                                   vec![Lit::U32(1_u32).into(); size.into()],
                                ).into(),
                        value : Some(Value::Vector(vec![1_u32.into(); size.into()])),
                    },
                    ScalarType::I32 => ConNode {
                        node : TypeConsExpr::new(
                                   data_type,
                                   vec![Lit::I32(1_i32).into(); size.into()],
                                ).into(),
                        value : Some(Value::Vector(vec![1_i32.into(); size.into()])),
                    },
                    ScalarType::F32 => ConNode {
                        node : TypeConsExpr::new(
                                   data_type,
                                   vec![Lit::F32(1_f32).into(); size.into()],
                                ).into(),
                        value : Some(Value::Vector(vec![1_f32.into(); size.into()])),
                    },
                    ScalarType::Bool => ConNode {
                        node : TypeConsExpr::new(
                                   data_type,
                                   vec![Lit::Bool(true).into(); size.into()],
                                ).into(),
                        value : Some(Value::Vector(vec![Value::Lit(Lit::Bool(true)); size.into()])),
                    },
                }
            },
            _ => {
                println!("data type: {data_type}");
                todo!();
            }
        }
        
    }

    fn evaluate_bin_op(
        &self,
        data_type : &DataType,
        op : &BinOp,
        l : Value,
        r : Value
    ) -> Option<Value> {
       
        match(l.clone(), r.clone()) {

            (Value::Vector(lv), Value::Vector(rv)) => {
                return self.eval_bin_op_vector(data_type, op, lv, rv);
            },

            (Value::Lit(lv), Value::Lit(rv)) => {
                return self.eval_bin_op_scalar(op, lv, rv);
            },
            // mixed scalar and vector binops require converting the 
            // scalar s to a vector<s> and performing component wise binop
            (Value::Lit(_), Value::Vector(rv)) => {
                let lv_vec = vec![l; rv.len()];

                return self.eval_bin_op_vector(data_type, op, lv_vec, rv.to_vec());
            },
            (Value::Vector(lv), Value::Lit(_)) => {
                let rv_vec = vec![r; lv.len()];

                return self.eval_bin_op_vector(data_type, op, lv.to_vec(), rv_vec);
            },
        }
    }
                
    fn eval_bin_op_scalar(
        &self,
        op : &BinOp,
        lv : Lit,
        rv : Lit
    ) -> Option<Value> {

        match op {
            /* BinOp shift concretization involves two checks
               (1) bit width of shiftee < value of shifter
               (2) no overflow
            */
            BinOp::LShift | BinOp::RShift => {
                return self.eval_bin_op_shift(op, lv, rv);
            },
            BinOp::Plus 
                | BinOp::Minus 
                | BinOp::Times 
                | BinOp::Divide
                | BinOp::Mod => {
                
                match (lv, rv) {

                    (Lit::I32(l_lit), Lit::I32(r_lit)) => { 
                        let result = binop_int_arith!(op, l_lit, r_lit);
                        return Value::from_i32(result);
                    }, 
                    (Lit::U32(l_lit), Lit::U32(r_lit)) => { 
                        let result = binop_int_arith!(op, l_lit, r_lit);
                        return Value::from_u32(result);
                    },
                    (Lit::F32(l_lit), Lit::F32(r_lit)) => { 
                        let result = binop_float(op, l_lit, r_lit);
                        return Value::from_f32(result);
                    },      
                    _ => {return None;},
                }
            },
            _ => { return None; },
        }
    }

    fn eval_bin_op_shift(
        &self,
        op : &BinOp,
        lv : Lit,
        rv : Lit,
    ) -> Option<Value> {

        // check condition 1
        let result = binop_int_shift!(op, lv, rv);

        if result.is_none() {
            return None;
        }

        // check condition 2
        match (lv, rv) {
            (Lit::I32(l), Lit::U32(r)) => {
                match op {
                    BinOp::LShift => {
                    
                        if l.leading_zeros() < (r + 1) 
                            && l.leading_ones() < (r + 1) {
                                return None;
                        }},
                    _ => (),
                }
            },
            (Lit::U32(l), Lit::U32(r)) => {
                match op {
                    BinOp::LShift => {
                        if l.leading_zeros() < r {
                            return None;
                        }},
                    _ => (),
                }
            },
            _=> todo!(),
        }

        // if we have passed all of the checks, return the result
        return result;
    }
               
    fn eval_bin_op_vector(
        &self,
        data_type : &DataType,
        op : &BinOp,
        l : Vec<Value>,
        r : Vec<Value>
    ) ->  Option<Value> {

        let mut result : Vec<Value> = Vec::new();
        
        for (a, b) in l.iter().zip(r.iter()) {

            let elem = match (a, b) {

                (Value::Lit(lv), Value::Lit(rv)) => {
                    self.eval_bin_op_scalar(op, *lv, *rv)
                }

                (Value::Vector(lv), Value::Vector(rv)) => {

                    self.eval_bin_op_vector(data_type, op, lv.to_vec(), rv.to_vec())
                },
                
                // rules for binop on mixed scalar and vector operands are that
                // the scalar s is first converted to a vector of matching size
                // that is filled with scalar s
                (Value::Vector(lv), Value::Lit(_)) => {
                    let rv_vec = vec![(*b).clone(); lv.len()];
                    
                    self.eval_bin_op_vector(data_type, op, lv.to_vec(), rv_vec)
                },
                (Value::Lit(_), Value::Vector(rv)) => {
                    let lv_vec = vec![(*a).clone(); rv.len()];
                    
                    self.eval_bin_op_vector(data_type, op, lv_vec, rv.to_vec())
                },
            };

            match elem {
                Some(e) => result.push(e),
                None => {return None;},
            }
        }

        return Some(Value::Vector(result));
    }

    fn concretize_unop(
        &self,
        data_type : DataType,
        op : UnOp,
        inner : ConNode
    ) -> ConNode {
        
        if inner.value.is_none() {
            return ConNode {
                node : UnOpExpr::new(op, inner.node).into(),
                value : None,
            }
        }

        let result = self.eval_unop(op, inner.clone());

        match result {
            Some(r) => ConNode {
                node : UnOpExpr::new(op, inner.node).into(),
                value : Some(r),
            },
            None => self.default_node(data_type),
        }
    }

    fn eval_unop(
        &self,
        op : UnOp,
        inner : ConNode
    ) -> Option<Value> {

        match inner.value.unwrap() {
            Value::Vector(v) => { return self.eval_unop_vector(op, v);},
            Value::Lit(v) => { return self.eval_unop_scalar(op, v);},
        }
    }

    fn eval_unop_scalar(
        &self,
        op : UnOp,
        inner : Lit
    ) -> Option<Value> {

        match op {
            UnOp::Neg => {
                match inner {
                    Lit::I32(i) => {
                        
                        // WGSL negation of  e : T where T is an 
                        // integer scalar and e evaluates to the 
                        // largest negative value, gives result e
                        if i == -2147483648 {
                            Value::from_i32(Some(i))
                        }
                        else {
                            Value::from_i32(Some(-i))
                        }
                    },
                    Lit::F32(f) => Value::from_f32(Some(-f)),
                    _ => {
                        panic!(); // can't negate other types
                    },
                }
            },
            UnOp::BitNot => {
                match inner {
                    Lit::I32(i) => Value::from_i32(Some(!i)),
                    Lit::U32(u) => Value::from_u32(Some(!u)),
                    _ => panic!(), // can't bitnot other types
                }
            }
            _ => None,
        }
    }

    fn eval_unop_vector(
        &self,
        op : UnOp,
        inner : Vec<Value>
    ) -> Option<Value> {

        let mut result : Vec<Value> = Vec::new();

        for i in inner {
            match i {
                Value::Lit(v) => {
                    let elem = self.eval_unop_scalar(op, v);
                    
                    match elem {
                        Some(e) => result.push(e.into()),
                        None => {return None;},
                    }
                },
                Value::Vector(v) => {
                    let elem = self.eval_unop_vector(op, v.to_vec());

                    match elem {
                        Some(e) => result.push(e.into()),
                        None => {return None;},
                    }

                },
            }
        }

        return Some(Value::Vector(result));
        
    }
}

