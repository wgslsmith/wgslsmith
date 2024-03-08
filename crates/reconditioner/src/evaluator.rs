use ast::*;

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
                initializer.map(|e| self.concretize_expr(e)),
            ).into(),
            Statement::Assignment(AssignmentStatement {lhs, op, rhs}) => {
                AssignmentStatement::new(
                    lhs,
                    op,
                    self.concretize_expr(rhs),
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
                    self.concretize_expr(condition),
                    body.into_iter().map(|s| self.concretize_stmt(s)).collect(),
                    )
                .with_else(else_.map(|els| *els))
                .into(),
            Statement::Return(ReturnStatement {value}) => ReturnStatement {
                value: value.map(|e| self.concretize_expr(e)),
            }.into(),
            Statement::Switch(SwitchStatement {
                selector,
                cases,
                default,
            }) => SwitchStatement::new(
                self.concretize_expr(selector),
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
                    .map(|e| self.concretize_expr(e))
                    .collect()
                ).into(),
            Statement::Loop(LoopStatement {body}) => LoopStatement::new(
                body.into_iter().map(|s| self.concretize_stmt(s)).collect()).into(),
            Statement::ForLoop(ForLoopStatement {header, body}) => ForLoopStatement::new(
                ForLoopHeader {
                    init : header.init.map(|init| self.concretize_for_init(init)),
                    condition: header.condition.map(|e| self.concretize_expr(e)),
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
               initializer.map(|e| self.concretize_expr(e)),
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
                    Postfix::Index(index) => Postfix::Index(self.concretize_expr(*index).into()),
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
           selector : concretized_selector,
           body : concretized_body,
       }
   }

    fn concretize_expr(&self, node: ExprNode) -> ExprNode {

        //TODO: if expr contains var, return (since not concretizable)

        let concretized = match node.expr {
            Expr::Lit(lit) => self.test_lit(lit).into(), // placeholder
            Expr::TypeCons(expr) => Expr::TypeCons(TypeConsExpr::new(
                expr.data_type,
                expr.args
                    .into_iter()
                    .map(|e| self.concretize_expr(e))
                    .collect()
            )),
            Expr::Var(expr) => todo!(),
            Expr::UnOp(expr) => todo!(),
            Expr::BinOp(expr) => todo!(),
            Expr::FnCall(expr) => todo!(),
            Expr::Postfix(expr) => todo!(),
        };

        ExprNode {
            data_type: node.data_type,
            expr : concretized,
        }
    }

    fn test_lit(&self, lit : Lit) -> Expr {
       
        //TODO: placeholder to test operation of concretization 
        let val = match lit {
            Lit::I32(_) => Lit::I32(1),
            Lit::U32(_) => Lit::U32(1),
            Lit::F32(_) => Lit::F32(1.0),
            e => e,
        };

        return Expr::Lit(val)
    }
}
