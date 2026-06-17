mod safe_wrappers;

pub mod analysis;
pub mod cli;

use std::collections::HashSet;
use std::fmt::Display;

use ast::types::{DataType, MemoryViewType, ScalarType};
use ast::*;

pub struct ReconditionResult {
    pub ast: Module,
    pub loop_count: u32,
}

#[derive(Hash, PartialEq, Eq)]
enum Wrapper {
    FloatOp(DataType),
    FloatDivide(DataType),
    Mod(DataType),
    Index(DataType),
}

impl Wrapper {
    fn gen_fn_decl(&self) -> FnDecl {
        let name = self.to_string();
        match self {
            Wrapper::FloatOp(ty) => safe_wrappers::float(name, ty),
            Wrapper::FloatDivide(ty) => safe_wrappers::float_divide(name, ty),
            Wrapper::Mod(ty) => safe_wrappers::modulo(name, ty),
            Wrapper::Index(ty) => safe_wrappers::index(name, ty),
        }
    }
}

impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, ty) = match self {
            Wrapper::FloatOp(ty) => ("f_op", ty),
            Wrapper::FloatDivide(ty) => ("div", ty),
            Wrapper::Mod(ty) => ("mod", ty),
            Wrapper::Index(ty) => ("index", ty),
        };

        write!(f, "_wgslsmith_{name}_")?;

        match ty {
            DataType::Scalar(ty) => write!(f, "{ty}"),
            DataType::Vector(n, ty) => write!(f, "vec{n}_{ty}"),
            _ => unimplemented!("no wrappers available for expressions of type `{ty}`"),
        }
    }
}

#[derive(Default)]
pub struct Options {
    pub only_loops: bool,
}

pub fn recondition(ast: Module) -> Module {
    recondition_with(ast, Options::default())
}

pub fn recondition_with(ast: Module, options: Options) -> Module {
    let mut reconditioner = Reconditioner::new(options);

    let mut ast = concretizer::concretize(ast);

    let functions = ast
        .functions
        .into_iter()
        .map(|f| reconditioner.recondition_fn(f))
        .collect::<Vec<_>>();

    ast.functions = reconditioner
        .wrappers
        .iter()
        .map(Wrapper::gen_fn_decl)
        .chain(functions)
        .collect();

    if reconditioner.loop_var > 0 {
        ast.vars.push(GlobalVarDecl {
            attrs: vec![],
            data_type: DataType::array(ScalarType::U32, Some(reconditioner.loop_var)),
            name: "LOOP_COUNTERS".into(),
            initializer: None,
            qualifier: Some(VarQualifier {
                storage_class: StorageClass::Private,
                access_mode: None,
            }),
        });
    }

    ast
}

struct Reconditioner {
    loop_var: u32,
    wrappers: HashSet<Wrapper>,
    only_loops: bool,
}

impl Reconditioner {
    fn new(options: Options) -> Reconditioner {
        Reconditioner {
            loop_var: 0,
            wrappers: HashSet::new(),
            only_loops: options.only_loops,
        }
    }

    fn recondition_fn(&mut self, mut decl: FnDecl) -> FnDecl {
        decl.body = decl
            .body
            .into_iter()
            .map(|s| self.recondition_stmt(s))
            .collect();
        decl
    }

    fn recondition_else(&mut self, els: Else) -> Else {
        match els {
            Else::If(IfStatement {
                condition,
                body,
                else_,
            }) => Else::If(IfStatement {
                condition: self.recondition_expr(condition),
                body: body.into_iter().map(|s| self.recondition_stmt(s)).collect(),
                else_: else_.map(|els| Box::new(self.recondition_else(*els))),
            }),
            Else::Else(stmts) => Else::Else(
                stmts
                    .into_iter()
                    .map(|s| self.recondition_stmt(s))
                    .collect(),
            ),
        }
    }

    fn recondition_stmt(&mut self, stmt: Statement) -> Statement {
        match stmt {
            Statement::LetDecl(LetDeclStatement { ident, initializer }) => {
                LetDeclStatement::new(ident, self.recondition_expr(initializer)).into()
            }
            Statement::VarDecl(VarDeclStatement {
                ident,
                data_type,
                initializer,
            }) => VarDeclStatement::new(
                ident,
                data_type,
                initializer.map(|e| self.recondition_expr(e)),
            )
            .into(),
            Statement::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                AssignmentStatement::new(
                    self.recondition_assignment_lhs(lhs),
                    op,
                    self.recondition_expr(rhs),
                )
                .into()
            }
            Statement::Compound(s) => {
                Statement::Compound(s.into_iter().map(|s| self.recondition_stmt(s)).collect())
            }
            Statement::If(IfStatement {
                condition,
                body,
                else_,
            }) => IfStatement::new(
                self.recondition_expr(condition),
                body.into_iter().map(|s| self.recondition_stmt(s)).collect(),
            )
            .with_else(else_.map(|els| self.recondition_else(*els)))
            .into(),
            Statement::Return(ReturnStatement { value }) => ReturnStatement {
                value: value.map(|e| self.recondition_expr(e)),
            }
            .into(),
            Statement::Loop(LoopStatement {
                mut body,
                continuing,
            }) => {
                if continuing.is_some() {
                    let last_decl_idx = body
                        .iter()
                        .rposition(|s| matches!(s, Statement::LetDecl(_) | Statement::VarDecl(_)));
                    if let Some(idx) = last_decl_idx {
                        for stmt in body.iter_mut().take(idx) {
                            Self::replace_continue(stmt);
                        }
                    }
                }

                LoopStatement::new(
                    self.recondition_loop_body(body),
                    continuing.map(|ContinuingBlock { stmts, break_if }| {
                        let new_stmts: Vec<Statement> = stmts
                            .into_iter()
                            .map(|s| self.recondition_stmt(s))
                            .collect();

                        ContinuingBlock {
                            stmts: new_stmts,
                            break_if: break_if.map(|e| self.recondition_expr(e)),
                        }
                    }),
                )
                .into()
            }
            Statement::While(stmt) => Statement::While(WhileStatement {
                condition: self.recondition_expr(stmt.condition),
                body: self.recondition_loop_body(stmt.body),
            }),
            Statement::Break => Statement::Break,
            Statement::Switch(SwitchStatement {
                selector,
                cases,
                default,
            }) => SwitchStatement::new(
                self.recondition_expr(selector),
                cases
                    .into_iter()
                    .map(|SwitchCase { selector, body }| SwitchCase {
                        selector: self.recondition_expr(selector),
                        body: body
                            .into_iter()
                            .map(|it| self.recondition_stmt(it))
                            .collect(),
                    })
                    .collect(),
                default
                    .into_iter()
                    .map(|it| self.recondition_stmt(it))
                    .collect(),
            )
            .into(),
            Statement::ForLoop(ForLoopStatement { header, body }) => ForLoopStatement::new(
                ForLoopHeader {
                    init: header.init.map(|init| self.recondition_for_init(init)),
                    condition: header.condition.map(|e| self.recondition_expr(e)),
                    update: header
                        .update
                        .map(|update| self.recondition_for_update(update)),
                },
                self.recondition_loop_body(body),
            )
            .into(),
            Statement::FnCall(FnCallStatement { ident, args }) => {
                Statement::FnCall(FnCallStatement::new(
                    ident,
                    args.into_iter()
                        .map(|it| self.recondition_expr(it))
                        .collect(),
                ))
            }
            Statement::Continue => Statement::Continue,
            Statement::Fallthrough => Statement::Fallthrough,
            Statement::Increment(IncrementStatement { lhs }) => {
                IncrementStatement::new(self.recondition_assignment_lhs(lhs)).into()
            }
            Statement::Decrement(DecrementStatement { lhs }) => {
                DecrementStatement::new(self.recondition_assignment_lhs(lhs)).into()
            }
        }
    }

    fn recondition_for_init(&mut self, init: ForLoopInit) -> ForLoopInit {
        match init {
            ForLoopInit::VarDecl(VarDeclStatement {
                ident,
                data_type,
                initializer,
            }) => ForLoopInit::VarDecl(VarDeclStatement::new(
                ident,
                data_type,
                initializer.map(|e| self.recondition_expr(e)),
            )),
            ForLoopInit::LetDecl(LetDeclStatement { ident, initializer }) => ForLoopInit::LetDecl(
                LetDeclStatement::new(ident, self.recondition_expr(initializer)),
            ),
            ForLoopInit::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                ForLoopInit::Assignment(AssignmentStatement::new(
                    self.recondition_assignment_lhs(lhs),
                    op,
                    self.recondition_expr(rhs),
                ))
            }
            ForLoopInit::Increment(IncrementStatement { lhs }) => ForLoopInit::Increment(
                IncrementStatement::new(self.recondition_assignment_lhs(lhs)),
            ),
            ForLoopInit::Decrement(DecrementStatement { lhs }) => ForLoopInit::Decrement(
                DecrementStatement::new(self.recondition_assignment_lhs(lhs)),
            ),
            ForLoopInit::Call(FnCallStatement { ident, args }) => {
                ForLoopInit::Call(FnCallStatement::new(
                    ident,
                    args.into_iter()
                        .map(|it| self.recondition_expr(it))
                        .collect(),
                ))
            }
        }
    }

    fn recondition_for_update(&mut self, update: ForLoopUpdate) -> ForLoopUpdate {
        match update {
            ForLoopUpdate::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                ForLoopUpdate::Assignment(AssignmentStatement::new(
                    self.recondition_assignment_lhs(lhs),
                    op,
                    self.recondition_expr(rhs),
                ))
            }
            ForLoopUpdate::Increment(IncrementStatement { lhs }) => ForLoopUpdate::Increment(
                IncrementStatement::new(self.recondition_assignment_lhs(lhs)),
            ),
            ForLoopUpdate::Decrement(DecrementStatement { lhs }) => ForLoopUpdate::Decrement(
                DecrementStatement::new(self.recondition_assignment_lhs(lhs)),
            ),
            ForLoopUpdate::Call(FnCallStatement { ident, args }) => {
                ForLoopUpdate::Call(FnCallStatement::new(
                    ident,
                    args.into_iter()
                        .map(|it| self.recondition_expr(it))
                        .collect(),
                ))
            }
        }
    }

    fn recondition_loop_body(&mut self, body: Vec<Statement>) -> Vec<Statement> {
        let id = self.loop_var();

        let counters_ty = DataType::Ref(MemoryViewType::new(
            DataType::array(ScalarType::U32, None),
            StorageClass::Private,
        ));

        let break_check = IfStatement::new(
            BinOpExpr::new(
                BinOp::GreaterEqual,
                PostfixExpr::new(
                    VarExpr::new("LOOP_COUNTERS").into_node(counters_ty.clone()),
                    Postfix::index(Lit::U32(id)),
                ),
                Lit::U32(1),
            ),
            vec![Statement::Break],
        );

        let counter_increment = AssignmentStatement::new(
            AssignmentLhs::array_index("LOOP_COUNTERS", counters_ty.clone(), Lit::U32(id).into()),
            AssignmentOp::Simple,
            BinOpExpr::new(
                BinOp::Plus,
                PostfixExpr::new(
                    VarExpr::new("LOOP_COUNTERS").into_node(counters_ty),
                    Postfix::index(Lit::U32(id)),
                ),
                Lit::U32(1),
            ),
        );

        std::iter::once(break_check.into())
            .chain(std::iter::once(counter_increment.into()))
            .chain(body.into_iter().map(|s| self.recondition_stmt(s)))
            .collect()
    }

    fn recondition_assignment_lhs(&mut self, lhs: AssignmentLhs) -> AssignmentLhs {
        if self.only_loops {
            return lhs;
        }

        match lhs {
            AssignmentLhs::Phony => AssignmentLhs::Phony,
            AssignmentLhs::Expr(expr) => AssignmentLhs::Expr(self.recondition_lhs_expr(expr)),
        }
    }

    fn recondition_lhs_expr(&mut self, node: LhsExprNode) -> LhsExprNode {
        let expr = match node.expr {
            LhsExpr::Ident(ident) => LhsExpr::Ident(ident),
            LhsExpr::Postfix(expr, postfix) => {
                let expr = Box::new(self.recondition_lhs_expr(*expr));
                let postfix = match postfix {
                    Postfix::Index(index) => {
                        let index = self.recondition_expr(*index);
                        Postfix::index(self.recondition_array_index(&expr.data_type, index))
                    }
                    Postfix::Member(ident) => Postfix::Member(ident),
                };

                LhsExpr::Postfix(expr, postfix)
            }
            LhsExpr::Deref(_) => todo!(),
            LhsExpr::AddressOf(_) => todo!(),
        };

        LhsExprNode { expr, ..node }
    }

    fn recondition_expr(&mut self, node: ExprNode) -> ExprNode {
        if self.only_loops {
            return node;
        }

        let reconditioned = match node.expr {
            Expr::TypeCons(expr) => Expr::TypeCons(TypeConsExpr::new(
                expr.data_type,
                expr.args
                    .into_iter()
                    .map(|e| self.recondition_expr(e))
                    .collect(),
            )),
            Expr::UnOp(expr) => {
                let inner = self.recondition_expr(*expr.inner);
                let op = expr.op;
                match op {
                    UnOp::Neg => {
                        let data_type = inner.data_type.dereference().clone();
                        let mut expr = UnOpExpr::new(UnOp::Neg, inner).into();
                        if !data_type.is_matrix()
                            && matches!(
                                data_type.as_scalar().unwrap(),
                                ScalarType::F32 | ScalarType::F16
                            )
                        {
                            expr = FnCallExpr::new(
                                self.safe_wrapper(Wrapper::FloatOp(data_type.clone())),
                                vec![ExprNode { data_type, expr }],
                            )
                            .into();
                        }
                        expr
                    }
                    _ => UnOpExpr::new(op, inner).into(),
                }
            }
            Expr::BinOp(expr) => {
                let left = self.recondition_expr(*expr.left);
                let right = self.recondition_expr(*expr.right);
                return self.recondition_bin_op_expr(node.data_type, expr.op, left, right);
            }
            Expr::FnCall(expr) => {
                let args: Vec<ExprNode> = expr
                    .args
                    .into_iter()
                    .map(|e| self.recondition_expr(e))
                    .collect();

                let expr = FnCallExpr::new(expr.ident, args);

                if !node.data_type.is_matrix()
                    && matches!(
                        node.data_type.as_scalar(),
                        Some(ScalarType::F32 | ScalarType::F16)
                    )
                {
                    FnCallExpr::new(
                        self.safe_wrapper(Wrapper::FloatOp(node.data_type.clone())),
                        vec![expr.into_node(node.data_type.clone())],
                    )
                    .into()
                } else {
                    expr.into()
                }
            }
            Expr::Postfix(expr) => {
                let e = self.recondition_expr(*expr.inner);
                let postfix = match expr.postfix {
                    Postfix::Index(index) => {
                        let index = self.recondition_expr(*index);
                        Postfix::Index(Box::new(self.recondition_array_index(&e.data_type, index)))
                    }
                    Postfix::Member(n) => Postfix::Member(n),
                };

                PostfixExpr::new(e, postfix).into()
            }
            e => e,
        };

        ExprNode {
            data_type: node.data_type,
            expr: reconditioned,
        }
    }

    fn recondition_array_index(&mut self, array_type: &DataType, index: ExprNode) -> ExprNode {
        let size = match array_type.dereference() {
            DataType::Array(_, Some(n)) => *n,
            DataType::Array(_, None) => {
                todo!("runtime-sized arrays are not currently supported")
            }
            DataType::Vector(n, _) => *n as u32,
            DataType::Matrix(c, _, _) => *c as u32,
            _ => unreachable!("index operator cannot be applied to type `{array_type}`"),
        };

        let index_type = index.data_type.dereference().clone();
        let size_expr = match index_type.as_scalar().unwrap() {
            ScalarType::I32 => Lit::I32(size as i32),
            ScalarType::U32 => Lit::U32(size),
            _ => unreachable!("index expression must be an integer"),
        };

        FnCallExpr::new(
            self.safe_wrapper(Wrapper::Index(index_type.clone())),
            vec![index, size_expr.into()],
        )
        .into_node(index_type)
    }

    fn recondition_shift_expr(
        &mut self,
        ty: DataType,
        shift_op: BinOp,
        operand: ExprNode,
        shift_value: ExprNode,
    ) -> ExprNode {
        let shift_type = shift_value.data_type.dereference();
        let shift_bound: ExprNode = match ty {
            DataType::Scalar(_) => Lit::U32(32).into(),
            DataType::Vector(_, _) => {
                TypeConsExpr::new(shift_type.clone(), vec![Lit::U32(32).into()]).into()
            }
            _ => unreachable!(),
        };

        ExprNode::from(BinOpExpr::new(
            shift_op,
            operand,
            BinOpExpr::new(BinOp::Mod, shift_value, shift_bound),
        ))
    }

    fn recondition_bin_op_expr(
        &mut self,
        data_type: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        if data_type.is_matrix() {
            return BinOpExpr::new(op, l, r).into();
        }

        if let BinOp::LShift | BinOp::RShift = op {
            return self.recondition_shift_expr(data_type, op, l, r);
        }

        match data_type.as_scalar().unwrap() {
            ScalarType::I32 | ScalarType::U32 => {
                self.recondition_integer_bin_op_expr(data_type, op, l, r)
            }
            ScalarType::F32 | ScalarType::F16 if op == BinOp::Divide => {
                self.recondition_floating_point_div_expr(data_type, op, l, r)
            }
            ScalarType::F32 | ScalarType::F16 => {
                self.recondition_floating_point_bin_op_expr(data_type, op, l, r)
            }
            ScalarType::Bool => BinOpExpr::new(op, l, r).into(),
        }
    }

    fn recondition_integer_bin_op_expr(
        &mut self,
        data_type: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        let name = match op {
            BinOp::Mod => self.safe_wrapper(Wrapper::Mod(data_type.clone())),
            op => return BinOpExpr::new(op, l, r).into(),
        };

        FnCallExpr::new(name, vec![l, r]).into_node(data_type)
    }

    fn recondition_floating_point_bin_op_expr(
        &mut self,
        data_type: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        FnCallExpr::new(
            self.safe_wrapper(Wrapper::FloatOp(data_type.clone())),
            vec![BinOpExpr::new(op, l, r).into()],
        )
        .into_node(data_type)
    }

    fn recondition_floating_point_div_expr(
        &mut self,
        data_type: DataType,
        op: BinOp,
        l: ExprNode,
        r: ExprNode,
    ) -> ExprNode {
        let wrapper = match op {
            BinOp::Divide => Wrapper::FloatDivide(data_type.clone()),
            _ => unreachable!(),
        };
        FnCallExpr::new(self.safe_wrapper(wrapper), vec![l, r]).into_node(data_type)
    }

    fn loop_var(&mut self) -> u32 {
        let cur = self.loop_var;
        self.loop_var += 1;
        cur
    }

    fn safe_wrapper(&mut self, wrapper: Wrapper) -> String {
        let ident = wrapper.to_string();
        self.wrappers.insert(wrapper);
        ident
    }

    fn replace_continue(stmt: &mut Statement) {
        match stmt {
            Statement::Continue => {
                *stmt = Statement::Break;
            }
            Statement::Compound(stmts) => {
                for s in stmts {
                    Self::replace_continue(s);
                }
            }
            Statement::If(stmt) => {
                for s in &mut stmt.body {
                    Self::replace_continue(s);
                }
                if let Some(else_) = &mut stmt.else_ {
                    Self::replace_continue_else(else_.as_mut());
                }
            }
            Statement::Switch(stmt) => {
                for case in &mut stmt.cases {
                    for s in &mut case.body {
                        Self::replace_continue(s);
                    }
                }
                for s in &mut stmt.default {
                    Self::replace_continue(s);
                }
            }
            _ => {}
        }
    }

    fn replace_continue_else(else_: &mut Else) {
        match else_ {
            Else::If(stmt) => {
                for s in &mut stmt.body {
                    Self::replace_continue(s);
                }
                if let Some(els) = &mut stmt.else_ {
                    Self::replace_continue_else(els.as_mut());
                }
            }
            Else::Else(stmts) => {
                for s in stmts {
                    Self::replace_continue(s);
                }
            }
        }
    }
}
