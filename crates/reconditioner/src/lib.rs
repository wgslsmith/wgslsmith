mod safe_wrappers;

pub mod analysis;

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
    Clamp(DataType),
    Dot(DataType),
    FloatOp(DataType),
    Plus(DataType),
    Minus(DataType),
    Times(DataType),
    Divide(DataType),
    Mod(DataType),
}

impl Wrapper {
    fn gen_fn_decl(&self) -> FnDecl {
        let name = self.to_string();
        match self {
            Wrapper::Clamp(ty) => safe_wrappers::clamp(name, ty),
            Wrapper::Dot(ty) => safe_wrappers::dot(name, ty),
            Wrapper::FloatOp(ty) => safe_wrappers::float(name, ty),
            Wrapper::Plus(ty) => safe_wrappers::plus(name, ty),
            Wrapper::Minus(ty) => safe_wrappers::minus(name, ty),
            Wrapper::Times(ty) => safe_wrappers::times(name, ty),
            Wrapper::Divide(ty) => safe_wrappers::divide(name, ty),
            Wrapper::Mod(ty) => safe_wrappers::modulo(name, ty),
        }
    }
}

impl Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, ty) = match self {
            Wrapper::Clamp(ty) => ("CLAMP", ty),
            Wrapper::Dot(ty) => ("DOT", ty),
            Wrapper::FloatOp(ty) => ("FLOAT_OP", ty),
            Wrapper::Plus(ty) => ("PLUS", ty),
            Wrapper::Minus(ty) => ("MINUS", ty),
            Wrapper::Times(ty) => ("TIMES", ty),
            Wrapper::Divide(ty) => ("DIVIDE", ty),
            Wrapper::Mod(ty) => ("MOD", ty),
        };

        write!(f, "SAFE_{}_", name)?;

        match ty {
            DataType::Scalar(ty) => write!(f, "{}", ty),
            DataType::Vector(n, ty) => write!(f, "vec{}_{}", n, ty),
            _ => unimplemented!("no wrappers available for expressions of type `{ty}`"),
        }
    }
}

pub fn recondition(mut ast: Module) -> Module {
    let mut reconditioner = Reconditioner::default();

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

    ast.consts.extend([
        GlobalConstDecl {
            name: "INT_MIN".to_owned(),
            data_type: ScalarType::I32.into(),
            initializer: ExprNode {
                data_type: ScalarType::I32.into(),
                expr: Expr::Lit(Lit::I32(i32::MIN)),
            },
        },
        GlobalConstDecl {
            name: "INT_MAX".to_owned(),
            data_type: ScalarType::I32.into(),
            initializer: ExprNode {
                data_type: ScalarType::I32.into(),
                expr: Expr::Lit(Lit::I32(i32::MAX)),
            },
        },
        GlobalConstDecl {
            name: "UINT_MAX".to_owned(),
            data_type: ScalarType::U32.into(),
            initializer: ExprNode {
                data_type: ScalarType::U32.into(),
                expr: Expr::Lit(Lit::U32(u32::MAX)),
            },
        },
    ]);

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

#[derive(Default)]
struct Reconditioner {
    loop_var: u32,
    wrappers: HashSet<Wrapper>,
}

impl Reconditioner {
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
            Statement::Loop(LoopStatement { body }) => {
                LoopStatement::new(self.recondition_loop_body(body)).into()
            }
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
                    init: header.init,
                    condition: header.condition.map(|e| self.recondition_expr(e)),
                    update: header.update,
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
                        Postfix::index(self.recondition_array_index(&expr.data_type, *index))
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
                        let mut expr = self.recondition_negation(inner);
                        if data_type.as_scalar().unwrap() == ScalarType::F32 {
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

                let expr = match expr.ident.as_str() {
                    "clamp" => FnCallExpr::new(
                        self.safe_wrapper(Wrapper::Clamp(args[0].data_type.dereference().clone())),
                        args,
                    ),
                    "dot" if args[0].data_type.is_integer() => FnCallExpr::new(
                        self.safe_wrapper(Wrapper::Dot(args[0].data_type.dereference().clone())),
                        args,
                    ),
                    _ => FnCallExpr::new(expr.ident, args),
                };

                if matches!(node.data_type.as_scalar(), Some(ScalarType::F32)) {
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
                        Postfix::Index(Box::new(self.recondition_array_index(&e.data_type, *index)))
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

    fn recondition_negation(&mut self, inner: ExprNode) -> Expr {
        // TODO: Workaround for bug in naga which generates incorrect code for double negation
        // expression: https://github.com/gfx-rs/naga/issues/1564.
        // We transform a double negation into a single negation which is multiplied by -1.

        fn should_recondition(expr: &Expr) -> bool {
            // Recondition if inner is a unary negation or a negative literal
            matches!(expr, Expr::UnOp(UnOpExpr { op: UnOp::Neg, .. }))
                || matches!(expr, Expr::Lit(Lit::I32(v)) if *v < 0)
                || matches!(expr, Expr::Lit(Lit::F32(v)) if *v < 0.0)
        }

        let data_type = inner.data_type.dereference();
        let scalar_ty = data_type.as_scalar().unwrap();

        if !should_recondition(&inner.expr) {
            return UnOpExpr::new(UnOp::Neg, inner).into();
        }

        let scalar_lit = match scalar_ty {
            ScalarType::I32 => Lit::I32(-1),
            ScalarType::F32 => Lit::F32(-1.0),
            _ => unreachable!("negation can only be applied to signed integers and floats"),
        };

        let neg_multiplier = TypeConsExpr::new(data_type.clone(), vec![scalar_lit.into()]);

        BinOpExpr::new(BinOp::Times, neg_multiplier, inner).into()
    }

    fn recondition_array_index(&mut self, array_type: &DataType, index: ExprNode) -> ExprNode {
        let len_expr: ExprNode = match array_type.dereference() {
            DataType::Array(_, Some(n)) => Lit::U32(*n).into(),
            DataType::Array(_, None) => {
                todo!("runtime-sized arrays are not currently supported")
            }
            _ => unreachable!("index operator cannot be applied to type `{array_type}`"),
        };

        self.recondition_expr(BinOpExpr::new(BinOp::Mod, index, len_expr).into())
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
        if let BinOp::LShift | BinOp::RShift = op {
            return self.recondition_shift_expr(data_type, op, l, r);
        }

        match data_type.as_scalar().unwrap() {
            ScalarType::I32 | ScalarType::U32 => {
                self.recondition_integer_bin_op_expr(data_type, op, l, r)
            }
            ScalarType::F32 => self.recondition_floating_point_bin_op_expr(data_type, op, l, r),
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
            BinOp::Plus => self.safe_wrapper(Wrapper::Plus(data_type.clone())),
            BinOp::Minus => self.safe_wrapper(Wrapper::Minus(data_type.clone())),
            BinOp::Times => self.safe_wrapper(Wrapper::Times(data_type.clone())),
            BinOp::Divide => self.safe_wrapper(Wrapper::Divide(data_type.clone())),
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
}
