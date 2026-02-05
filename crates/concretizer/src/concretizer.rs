use crate::builtin::*;
use crate::helper;
use crate::value::*;
use ast::*;
use std::cmp::PartialEq;

// Concretized node struct contains a concretized node and
// the value from evaluating that node. The value is None
// if the node cannot be evaluated at shader creation time
// (i.e. the node is not a const expression, for example if
// it contains a runtime variable).

macro_rules! binop_int_arith {
    ($op:expr, $l:expr, $r:expr) => {
        match $op {
            BinOp::Plus => Some(($l).wrapping_add($r)),
            BinOp::Minus => Some(($l).wrapping_sub($r)),
            BinOp::Times => Some(($l).wrapping_mul($r)),
            BinOp::Divide => {
                if $r == 0 {
                    None
                } else {
                    Some(($l).wrapping_div($r))
                }
            }
            BinOp::Mod => {
                if $r == 0 {
                    None
                } else {
                    Some(($l).wrapping_rem($r))
                }
            }
            _ => None,
        }
    };
}

// WGSL shift evaluation rules involve two checks:
// e1 << e2 : T
// (1) bit width of e1 must be >= value of e2
// (2) result must not overflow
// analagous for >>
// Macro performs first check for bit width

fn binop_int_shift(op: &BinOp, l: Lit, r: Lit) -> Option<Value> {
    // rust checked_shl and checked_shr do not
    // include overflow checks
    match (l, r) {
        (Lit::I32(l), Lit::U32(r)) => match op {
            BinOp::LShift => Value::from_i32(l.checked_shl(r)),
            BinOp::RShift => Value::from_i32(l.checked_shr(r)),
            _ => None,
        },
        (Lit::U32(l), Lit::U32(r)) => match op {
            BinOp::LShift => Value::from_u32(l.checked_shl(r)),
            BinOp::RShift => Value::from_u32(l.checked_shr(r)),
            _ => None,
        },
        (Lit::I32(l), Lit::I32(r)) => {
            if r < 0 {
                return None;
            }
            let r = r as u32;
            match op {
                BinOp::LShift => Value::from_i32(l.checked_shl(r)),
                BinOp::RShift => Value::from_i32(l.checked_shr(r)),
                _ => None,
            }
        }
        (Lit::U32(l), Lit::I32(r)) => {
            if r < 0 {
                return None;
            }
            let r = r as u32;
            match op {
                BinOp::LShift => Value::from_u32(l.checked_shl(r)),
                BinOp::RShift => Value::from_u32(l.checked_shr(r)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn binop_float(op: &BinOp, l: f32, r: f32) -> Option<f32> {
    let result = match op {
        BinOp::Plus => l + r,
        BinOp::Minus => l - r,
        BinOp::Times => l * r,
        BinOp::Divide => l / r,
        BinOp::Mod => l % r,
        _ => todo!(),
    };

    in_float_range(result)

    // TODO: add float division and mod check
}

pub(super) fn in_float_range(f: f32) -> Option<f32> {
    if !(0.1_f32..=16777216_f32).contains(&f.abs()) {
        None
    } else {
        Some(f)
    }
}

#[derive(Clone)]
/// A concretized ExprNode. If the node is a constant expression, its value is stored in `value`.
struct ConcreteNode {
    node: ExprNode,
    pub value: Option<Value>,
}

// from ExprNode to ConcreteNode (for passing into concretize fns)
impl From<ExprNode> for ConcreteNode {
    fn from(node: ExprNode) -> Self {
        ConcreteNode { node, value: None }
    }
}

// from ConcreteNode to ExprNode (for extracting expr to rebuild program, when we don't care about values)
impl From<ConcreteNode> for ExprNode {
    fn from(con: ConcreteNode) -> Self {
        ExprNode {
            data_type: con.node.data_type,
            expr: con.node.expr,
        }
    }
}

#[derive(Default)]
pub struct Options {
    pub error_handling: ErrorHandling,
}

#[derive(Default, PartialEq)]
pub enum ErrorHandling {
    #[default]
    ReplaceWithDefault,
    Panic,
}

use std::collections::HashMap;

#[derive(Default)]
pub struct Concretizer {
    error_handling: ErrorHandling,
    // keep track of consts as we traverse the AST
    global_constants: HashMap<String, Value>,
    local_scopes: Vec<HashMap<String, Value>>,
}

impl Concretizer {
    pub fn new(options: Options) -> Concretizer {
        Concretizer {
            error_handling: options.error_handling,
            global_constants: HashMap::new(),
            local_scopes: Vec::new(),
        }
    }

    pub fn register_global_consts(&mut self, consts: &[GlobalConstDecl]) {
        for decl in consts {
            let con_node = self.concretize_expr(decl.initializer.clone());
            if let Some(val) = con_node.value {
                self.global_constants.insert(decl.name.clone(), val);
            }
        }
    }

    #[allow(dead_code)]
    // This will be useful once we add consts to the AST
    fn register_const(&mut self, name: String, val: Value) {
        self.local_scopes.last_mut().unwrap().insert(name, val);
    }

    fn enter_scope(&mut self) {
        self.local_scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.local_scopes.pop();
    }

    #[allow(dead_code)]
    // This will be useful once we add consts to the AST
    fn insert_const(&mut self, name: String, val: Value) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, val);
        }
    }

    fn lookup_const(&self, name: &str) -> Option<Value> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        self.global_constants.get(name).cloned()
    }

    pub(crate) fn concretize_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        self.local_scopes.clear();
        self.enter_scope();

        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.concretize_stmt(s))
            .collect();

        self.exit_scope();

        decl
    }

    fn concretize_stmt(&mut self, stmt: Statement) -> Statement {
        match stmt {
            Statement::LetDecl(LetDeclStatement { ident, initializer }) => {
                let con_init = self.concretize_expr(initializer);
                LetDeclStatement::new(ident, con_init).into()
            }
            Statement::VarDecl(VarDeclStatement {
                ident,
                data_type,
                initializer,
            }) => VarDeclStatement::new(
                ident,
                data_type,
                initializer.map(|e| self.concretize_expr(e).into()),
            )
            .into(),
            Statement::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                AssignmentStatement::new(lhs, op, self.concretize_expr(rhs)).into()
            }
            Statement::Compound(s) => {
                self.enter_scope();
                let stmts = s.into_iter().map(|s| self.concretize_stmt(s)).collect();
                self.exit_scope();
                Statement::Compound(stmts)
            }
            Statement::If(IfStatement {
                condition,
                body,
                else_,
            }) => {
                let cond = self.concretize_expr(condition);
                self.enter_scope();
                let new_body = body.into_iter().map(|s| self.concretize_stmt(s)).collect();
                self.exit_scope();

                let new_else = else_
                    .map(|els| match *els {
                        Else::If(stmt) => Else::If(match self.concretize_stmt(stmt.into()) {
                            Statement::If(s) => s,
                            _ => unreachable!(),
                        }),
                        Else::Else(stmts) => {
                            self.enter_scope();
                            let new_stmts =
                                stmts.into_iter().map(|s| self.concretize_stmt(s)).collect();
                            self.exit_scope();
                            Else::Else(new_stmts)
                        }
                    })
                    .map(Box::new);

                IfStatement::new(cond, new_body)
                    .with_else(new_else.map(|e| *e))
                    .into()
            }
            Statement::Return(ReturnStatement { value }) => ReturnStatement {
                value: value.map(|e| self.concretize_expr(e).into()),
            }
            .into(),
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
                {
                    self.enter_scope();
                    let def = default
                        .into_iter()
                        .map(|s| self.concretize_stmt(s))
                        .collect();
                    self.exit_scope();
                    def
                },
            )
            .into(),
            Statement::FnCall(FnCallStatement { ident, args }) => FnCallStatement::new(
                ident,
                args.into_iter()
                    .map(|e| self.concretize_expr(e).into())
                    .collect(),
            )
            .into(),
            Statement::Loop(LoopStatement { body }) => {
                self.enter_scope();
                let new_body = body.into_iter().map(|s| self.concretize_stmt(s)).collect();
                self.exit_scope();
                LoopStatement::new(new_body).into()
            }
            Statement::ForLoop(ForLoopStatement { header, body }) => {
                self.enter_scope();
                let new_header = ForLoopHeader {
                    init: header.init.map(|init| self.concretize_for_init(init)),
                    condition: header.condition.map(|e| self.concretize_expr(e).into()),
                    update: header
                        .update
                        .map(|update| self.concretize_for_update(update)),
                };

                let new_body = body.into_iter().map(|s| self.concretize_stmt(s)).collect();
                self.exit_scope();

                ForLoopStatement::new(new_header, new_body).into()
            }
            Statement::Break => Statement::Break,
            Statement::Continue => Statement::Continue,
            Statement::Fallthrough => Statement::Fallthrough,
        }
    }

    fn concretize_for_init(&mut self, init: ForLoopInit) -> ForLoopInit {
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

    fn concretize_for_update(&mut self, update: ForLoopUpdate) -> ForLoopUpdate {
        match update {
            ForLoopUpdate::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                ForLoopUpdate::Assignment(AssignmentStatement::new(
                    self.concretize_assignment_lhs(lhs),
                    op,
                    self.concretize_expr(rhs),
                ))
            }
        }
    }

    fn concretize_assignment_lhs(&mut self, lhs: AssignmentLhs) -> AssignmentLhs {
        match lhs {
            AssignmentLhs::Phony => AssignmentLhs::Phony,
            AssignmentLhs::Expr(expr) => AssignmentLhs::Expr(self.concretize_lhs_expr(expr)),
        }
    }

    fn concretize_lhs_expr(&mut self, node: LhsExprNode) -> LhsExprNode {
        let expr = match node.expr {
            LhsExpr::Ident(ident) => LhsExpr::Ident(ident),
            LhsExpr::Postfix(expr, postfix) => LhsExpr::Postfix(
                self.concretize_lhs_expr(*expr).into(),
                match postfix {
                    Postfix::Index(index) => {
                        Postfix::Index(Box::new(self.concretize_expr(*index).into()))
                    }
                    Postfix::Member(string) => Postfix::Member(string),
                },
            ),
            LhsExpr::Deref(_) => todo!(),
            LhsExpr::AddressOf(_) => todo!(),
        };

        LhsExprNode {
            data_type: node.data_type,
            expr,
        }
    }

    fn concretize_switch_case(&mut self, case: SwitchCase) -> SwitchCase {
        let concretized_selector = self.concretize_expr(case.selector);

        self.enter_scope();
        let concretized_body = case
            .body
            .into_iter()
            .map(|s| self.concretize_stmt(s))
            .collect();
        self.exit_scope();

        SwitchCase {
            selector: concretized_selector.into(),
            body: concretized_body,
        }
    }

    fn concretize_expr(&mut self, node: ExprNode) -> ConcreteNode {
        //TODO: if expr contains var, return (since not concretizable)

        match node.expr {
            Expr::Lit(lit) => self.concretize_lit(lit),
            Expr::TypeCons(expr) => self.concretize_typecons(node.data_type, expr),
            Expr::UnOp(expr) => {
                let con_inner = self.concretize_expr(*expr.inner);

                self.concretize_unop(node.data_type, expr.op, con_inner)
            }
            Expr::BinOp(expr) => {
                let left = self.concretize_expr(*expr.left);
                let right = self.concretize_expr(*expr.right);

                self.concretize_bin_op(node.data_type, expr.op, left, right)
            }
            Expr::FnCall(expr) => {
                let concrete_args = expr
                    .args
                    .into_iter()
                    .map(|e| self.concretize_expr(e))
                    .collect();

                self.concretize_fncall(node.data_type, expr.ident, concrete_args)
            }
            Expr::Postfix(expr) => {
                let concrete_inner = self.concretize_expr(*expr.inner);

                let concrete_postfix = match expr.postfix {
                    Postfix::Index(index) => {
                        Postfix::Index(Box::new(self.concretize_expr(*index).into()))
                    }
                    Postfix::Member(string) => Postfix::Member(string),
                };

                ConcreteNode {
                    node: PostfixExpr::new(concrete_inner, concrete_postfix).into(),
                    value: None,
                }
            }
            Expr::Var(expr) => {
                let value = self.lookup_const(&expr.ident);
                ConcreteNode {
                    node: ExprNode {
                        data_type: node.data_type,
                        expr: Expr::Var(expr),
                    },
                    value,
                }
            }
        }
    }

    fn concretize_fncall(
        &self,
        data_type: DataType,
        ident: String,
        args: Vec<ConcreteNode>,
    ) -> ConcreteNode {
        // nodes : Vec<ExprNode>, vals : Option<Value>
        let (nodes, vals) = self.decompose_vec_con(args);

        // return node with none if any vals are none
        if self.contains_none(&vals) {
            if helper::is_invalid_bits_call(&ident, &vals) {
                return self.default_node(data_type);
            }

            if ident == "clamp" {
                if let (Some(Some(low)), Some(Some(high))) = (vals.get(1), vals.get(2)) {
                    if helper::is_invalid_clamp_bounds(low, high) {
                        return self.default_node(data_type);
                    }
                }
            }

            return ConcreteNode {
                node: FnCallExpr { ident, args: nodes }.into_node(data_type),
                value: None,
            };
        }

        let function = Builtin::convert(ident.clone());

        match function {
            Some(f) => {
                let evaluated_val = evaluate_builtin(&f, vals);

                match evaluated_val {
                    Some(_) => ConcreteNode {
                        node: FnCallExpr::new(ident, nodes).into_node(data_type),
                        value: evaluated_val,
                    },
                    None => self.default_node(data_type),
                }
            }
            None => ConcreteNode {
                node: FnCallExpr::new(ident, nodes).into_node(data_type),
                value: None,
            },
        }
    }

    fn contains_none(&self, vals: &[Option<Value>]) -> bool {
        vals.iter().any(|v| v.is_none())
    }

    fn concretize_typecons(&mut self, data_type: DataType, expr: TypeConsExpr) -> ConcreteNode {
        let concrete_args: Vec<ConcreteNode> = expr
            .args
            .into_iter()
            .map(|e| self.concretize_expr(e))
            .collect();

        let (new_node, new_val) = self.decompose_vec_con(concrete_args);

        let new_val = if self.contains_none(&new_val) {
            None
        } else {
            Some(Value::Vector(
                new_val.into_iter().map(|v| v.unwrap()).collect(),
            ))
        };

        ConcreteNode {
            node: TypeConsExpr::new(data_type, new_node).into(),
            value: new_val,
        }
    }

    fn decompose_vec_con(&self, vec: Vec<ConcreteNode>) -> (Vec<ExprNode>, Vec<Option<Value>>) {
        let mut new_node: Vec<ExprNode> = Vec::new();
        let mut new_val: Vec<Option<Value>> = Vec::new();

        for ConcreteNode { node, value } in vec {
            new_node.push(node);
            new_val.push(value);
        }

        (new_node, new_val)
    }

    fn concretize_lit(&self, lit: Lit) -> ConcreteNode {
        ConcreteNode {
            node: ExprNode {
                data_type: lit.data_type(),
                expr: Expr::Lit(lit),
            },
            value: Some(Value::Lit(lit)),
        }
    }

    fn concretize_bin_op(
        &self,
        data_type: DataType,
        op: BinOp,
        left: ConcreteNode,
        right: ConcreteNode,
    ) -> ConcreteNode {
        // if either left or right is not a const-expression, then
        // this node is not a const-expression
        if left.value.is_none() || right.value.is_none() {
            // If only right is a const-expression, validation could still detect div/mod by 0.
            if let Some(r_val) = &right.value {
                if helper::is_zero(r_val) {
                    match op {
                        BinOp::Divide => {
                            return ConcreteNode {
                                node: left.node,
                                value: None,
                            }
                        }
                        BinOp::Mod => return self.default_node(data_type),
                        _ => {}
                    }
                }
            }

            return ConcreteNode {
                node: ExprNode {
                    data_type,
                    expr: Expr::BinOp(BinOpExpr::new(op, left, right)),
                },
                value: None,
            };
        }

        let value: Option<Value> = self.evaluate_bin_op(
            &op,
            left.value.clone().unwrap(),
            right.value.clone().unwrap(),
        );

        if value.is_none() {
            self.default_node(data_type)
        } else {
            ConcreteNode {
                node: ExprNode {
                    data_type,
                    expr: Expr::BinOp(BinOpExpr::new(op, left, right)),
                },
                value,
            }
        }
    }

    fn default_node(&self, data_type: DataType) -> ConcreteNode {
        if self.error_handling == ErrorHandling::Panic {
            panic!("Invalid expression")
        }
        match data_type {
            DataType::Scalar(ty) => match ty {
                ScalarType::U32 => ConcreteNode {
                    node: Lit::U32(1_u32).into(),
                    value: Value::from_u32(Some(1_u32)),
                },
                ScalarType::I32 => ConcreteNode {
                    node: Lit::I32(1_i32).into(),
                    value: Value::from_i32(Some(1_i32)),
                },
                ScalarType::F32 => ConcreteNode {
                    node: Lit::F32(1_f32).into(),
                    value: Value::from_f32(Some(1_f32)),
                },
                ScalarType::Bool => ConcreteNode {
                    node: Lit::Bool(true).into(),
                    value: Value::from_bool(Some(true)),
                },
            },
            DataType::Vector(size, ty) => match ty {
                ScalarType::U32 => ConcreteNode {
                    node: TypeConsExpr::new(data_type, vec![Lit::U32(1_u32).into(); size.into()])
                        .into(),
                    value: Some(Value::Vector(vec![1_u32.into(); size.into()])),
                },
                ScalarType::I32 => ConcreteNode {
                    node: TypeConsExpr::new(data_type, vec![Lit::I32(1_i32).into(); size.into()])
                        .into(),
                    value: Some(Value::Vector(vec![1_i32.into(); size.into()])),
                },
                ScalarType::F32 => ConcreteNode {
                    node: TypeConsExpr::new(data_type, vec![Lit::F32(1_f32).into(); size.into()])
                        .into(),
                    value: Some(Value::Vector(vec![1_f32.into(); size.into()])),
                },
                ScalarType::Bool => ConcreteNode {
                    node: TypeConsExpr::new(data_type, vec![Lit::Bool(true).into(); size.into()])
                        .into(),
                    value: Some(Value::Vector(vec![
                        Value::Lit(Lit::Bool(true));
                        size.into()
                    ])),
                },
            },
            _ => {
                println!("data type: {data_type}");
                todo!();
            }
        }
    }

    fn evaluate_bin_op(&self, op: &BinOp, l: Value, r: Value) -> Option<Value> {
        match (l.clone(), r.clone()) {
            (Value::Vector(lv), Value::Vector(rv)) => self.eval_bin_op_vector(op, lv, rv),

            (Value::Lit(lv), Value::Lit(rv)) => self.eval_bin_op_scalar(op, lv, rv),
            (Value::Lit(_), Value::Vector(rv)) => {
                let lv_vec = vec![l; rv.len()];

                self.eval_bin_op_vector(op, lv_vec, rv.to_vec())
            }
            (Value::Vector(lv), Value::Lit(_)) => {
                let rv_vec = vec![r; lv.len()];

                self.eval_bin_op_vector(op, lv.to_vec(), rv_vec)
            }
        }
    }

    fn eval_bin_op_scalar(&self, op: &BinOp, lv: Lit, rv: Lit) -> Option<Value> {
        match op {
            // BinOp shift concretization involves two checks
            // (1) bit width of shiftee < value of shifter
            // (2) no overflow
            BinOp::LShift | BinOp::RShift => self.eval_bin_op_shift(op, lv, rv),
            BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Divide | BinOp::Mod => {
                match (lv, rv) {
                    (Lit::I32(l_lit), Lit::I32(r_lit)) => {
                        let result = binop_int_arith!(op, l_lit, r_lit);
                        Value::from_i32(result)
                    }
                    (Lit::U32(l_lit), Lit::U32(r_lit)) => {
                        let result = binop_int_arith!(op, l_lit, r_lit);
                        Value::from_u32(result)
                    }
                    (Lit::F32(l_lit), Lit::F32(r_lit)) => {
                        let result = binop_float(op, l_lit, r_lit);
                        Value::from_f32(result)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn eval_bin_op_shift(&self, op: &BinOp, lv: Lit, rv: Lit) -> Option<Value> {
        // check condition 1
        let result = binop_int_shift(op, lv, rv);

        result.as_ref()?;

        // check condition 2
        match (lv, rv) {
            (Lit::I32(l), Lit::U32(r)) => {
                if op == &BinOp::LShift
                    && (l.leading_zeros() < (r + 1) && l.leading_ones() < (r + 1))
                {
                    return None;
                }
            }
            (Lit::U32(l), Lit::U32(r)) => {
                if op == &BinOp::LShift && l.leading_zeros() < r {
                    return None;
                }
            }
            (Lit::I32(l), Lit::I32(r)) => {
                let r = r as u32;
                if op == &BinOp::LShift
                    && (l.leading_zeros() < (r + 1) && l.leading_ones() < (r + 1))
                {
                    return None;
                }
            }
            (Lit::U32(l), Lit::I32(r)) => {
                let r = r as u32;
                if op == &BinOp::LShift && l.leading_zeros() < r {
                    return None;
                }
            }
            _ => todo!(),
        }

        // if we have passed all of the checks, return the result
        result
    }

    fn eval_bin_op_vector(&self, op: &BinOp, l: Vec<Value>, r: Vec<Value>) -> Option<Value> {
        let mut result: Vec<Value> = Vec::new();

        for (a, b) in l.iter().zip(r.iter()) {
            let elem = match (a, b) {
                (Value::Lit(lv), Value::Lit(rv)) => self.eval_bin_op_scalar(op, *lv, *rv),

                (Value::Vector(lv), Value::Vector(rv)) => {
                    self.eval_bin_op_vector(op, lv.to_vec(), rv.to_vec())
                }

                // rules for binop on mixed scalar and vector operands are that
                // the scalar s is first converted to a vector of matching size
                // that is filled with scalar s
                (Value::Vector(lv), Value::Lit(_)) => {
                    let rv_vec = vec![(*b).clone(); lv.len()];

                    self.eval_bin_op_vector(op, lv.to_vec(), rv_vec)
                }
                (Value::Lit(_), Value::Vector(rv)) => {
                    let lv_vec = vec![(*a).clone(); rv.len()];

                    self.eval_bin_op_vector(op, lv_vec, rv.to_vec())
                }
            };

            match elem {
                Some(e) => result.push(e),
                None => {
                    return None;
                }
            }
        }

        Some(Value::Vector(result))
    }

    fn concretize_unop(&self, data_type: DataType, op: UnOp, inner: ConcreteNode) -> ConcreteNode {
        if inner.value.is_none() {
            return ConcreteNode {
                node: UnOpExpr::new(op, inner.node).into(),
                value: None,
            };
        }

        let result = self.eval_unop(op, inner.clone());

        match result {
            Some(r) => ConcreteNode {
                node: UnOpExpr::new(op, inner.node).into(),
                value: Some(r),
            },
            None => self.default_node(data_type),
        }
    }

    fn eval_unop(&self, op: UnOp, inner: ConcreteNode) -> Option<Value> {
        match inner.value.unwrap() {
            Value::Vector(v) => self.eval_unop_vector(op, v),
            Value::Lit(v) => self.eval_unop_scalar(op, v),
        }
    }

    fn eval_unop_scalar(&self, op: UnOp, inner: Lit) -> Option<Value> {
        match op {
            UnOp::Neg => {
                match inner {
                    Lit::I32(i) => {
                        if i == -2147483648 {
                            Value::from_i32(Some(i))
                        } else {
                            Value::from_i32(Some(-i))
                        }
                    }
                    Lit::F32(f) => Value::from_f32(Some(-f)),
                    _ => {
                        panic!(); // can't negate other types
                    }
                }
            }
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

    fn eval_unop_vector(&self, op: UnOp, inner: Vec<Value>) -> Option<Value> {
        let mut result: Vec<Value> = Vec::new();

        for i in inner {
            match i {
                Value::Lit(v) => {
                    let elem = self.eval_unop_scalar(op, v);

                    match elem {
                        Some(e) => result.push(e),
                        None => {
                            return None;
                        }
                    }
                }
                Value::Vector(v) => {
                    let elem = self.eval_unop_vector(op, v.to_vec());

                    match elem {
                        Some(e) => result.push(e),
                        None => {
                            return None;
                        }
                    }
                }
            }
        }

        Some(Value::Vector(result))
    }
}
