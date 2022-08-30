use ast::types::*;
use ast::*;
use std::rc::Rc;

pub fn generate_ub(ub_struct: Rc<StructDecl>, arr_type: DataType) -> Statement {
    let ub_arr_index = "_wgslsmith_ub_index";
    let ub_arr_index_expr_node = ExprNode {
        data_type: DataType::Scalar(ScalarType::U32),
        expr: Expr::Var(VarExpr::new("_wgslsmith_ub_index")),
    };
    let block: Vec<Statement> = vec![Statement::ForLoop(ForLoopStatement::new(
        ForLoopHeader {
            init: Some(ForLoopInit::VarDecl(VarDeclStatement::new(
                ub_arr_index,
                Some(DataType::Scalar(ScalarType::U32)),
                Some(ExprNode {
                    data_type: DataType::Scalar(ScalarType::U32),
                    expr: Expr::Postfix(PostfixExpr::new(
                        VarExpr::new("_wgslsmith_ub")
                            .into_node(DataType::Struct(ub_struct.clone())),
                        Postfix::Member("min_index".into()),
                    )),
                }),
            ))),
            condition: Some(ExprNode::from(BinOpExpr::new(
                BinOp::LessEqual,
                ub_arr_index_expr_node.clone(),
                ExprNode {
                    data_type: DataType::Scalar(ScalarType::U32),
                    expr: Expr::Postfix(PostfixExpr::new(
                        VarExpr::new("_wgslsmith_ub")
                            .into_node(DataType::Struct(ub_struct.clone())),
                        Postfix::Member("max_index".into()),
                    )),
                },
            ))),
            update: Some(ForLoopUpdate::Assignment(AssignmentStatement {
                lhs: AssignmentLhs::Expr(LhsExprNode::name(
                    format!("({})", ub_arr_index).into(),
                    DataType::Scalar(ScalarType::U32),
                )),
                op: AssignmentOp::Plus,
                rhs: ExprNode::from(Lit::U32(1)),
            })),
        },
        vec![Statement::Assignment(AssignmentStatement::new(
            AssignmentLhs::array_index(
                "_wgslsmith_ub_arr",
                DataType::Ref(MemoryViewType {
                    inner: Rc::new(arr_type),
                    storage_class: StorageClass::Storage,
                    access_mode: AccessMode::ReadWrite,
                }),
                ub_arr_index_expr_node,
            ),
            AssignmentOp::Simple,
            ExprNode {
                data_type: DataType::Vector(4, ScalarType::U32),
                expr: Expr::Postfix(PostfixExpr::new(
                    VarExpr::new("_wgslsmith_ub").into_node(DataType::Struct(ub_struct.clone())),
                    Postfix::Member("write_value".into()),
                )),
            },
        ))],
    ))];
    Statement::Compound(block)
}
