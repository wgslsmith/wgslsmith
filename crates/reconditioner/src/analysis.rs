use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ast::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum AccessType {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum RootIdentifier {
    Mem(u32),
    Param(u32),
}

impl RootIdentifier {
    fn map_param(self, mapper: impl FnOnce(u32) -> RootIdentifier) -> RootIdentifier {
        match self {
            RootIdentifier::Mem(_) => self,
            RootIdentifier::Param(p) => mapper(p),
        }
    }
}

#[derive(Clone, Debug)]
struct Scope<'a> {
    idents: HashMap<&'a str, RootIdentifier>,
}

#[derive(Debug)]
struct FnContext<'a> {
    name: &'a str,
    accesses: HashSet<(AccessType, RootIdentifier)>,
}

#[derive(Debug)]
struct FnCall<'a>(u32, &'a str, Vec<Option<RootIdentifier>>);

impl<'a> Hash for FnCall<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> PartialEq for FnCall<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for FnCall<'a> {}

#[derive(Debug, Default)]
struct Analysis<'a> {
    next_id: u32,
    next_mem_loc: u32,
    accesses: HashMap<&'a str, HashSet<(AccessType, RootIdentifier)>>,
    points_to: HashMap<&'a str, HashMap<u32, HashSet<u32>>>,
    calls: HashMap<&'a str, HashSet<FnCall<'a>>>,
}

impl<'a> Analysis<'a> {
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn next_mem_loc(&mut self) -> u32 {
        let loc = self.next_mem_loc;
        self.next_mem_loc += 1;
        loc
    }
}

/// PRECONDITION: Functions must be defined in order, such that earlier functions do not contain
/// calls to later functions.
pub fn analyse(module: &Module) -> bool {
    let mut analysis = Analysis::default();

    let mut global_scope = Scope {
        idents: Default::default(),
    };

    for var in &module.vars {
        global_scope
            .idents
            .insert(&var.name, RootIdentifier::Mem(analysis.next_mem_loc()));
    }

    for func in module.functions.iter().rev() {
        let accesses = visit_function(&mut analysis, global_scope.clone(), func);
        analysis.accesses.insert(&func.name, accesses);
    }

    // println!("{analysis:#?}");

    let Analysis {
        mut accesses,
        points_to,
        calls,
        ..
    } = analysis;

    for func in module.functions.iter() {
        // Expand the calls to determine the full set of possible memory accesses
        for call in calls.get(func.name.as_str()).into_iter().flatten() {
            if let Some(call_accesses) = accesses
                .get(call.1)
                .map(|it| it as *const HashSet<(AccessType, RootIdentifier)>)
            {
                // This should be safe as long as func.name != call.1 (which shouldn't be the
                // case since recursion is forbidden in WGSL)
                let call_accesses = unsafe { &*call_accesses };
                for (access_type, root_id) in call_accesses {
                    accesses.entry(func.name.as_str()).or_default().insert((
                        *access_type,
                        root_id.map_param(|it| call.2[it as usize].unwrap()),
                    ));
                }
            }
        }

        let mut expanded_accesses: HashMap<u32, HashSet<(AccessType, RootIdentifier)>> =
            HashMap::new();

        for (access_type, root_id) in accesses.get(func.name.as_str()).unwrap() {
            match root_id {
                RootIdentifier::Mem(loc) => {
                    expanded_accesses
                        .entry(*loc)
                        .or_default()
                        .insert((*access_type, *root_id));
                }
                RootIdentifier::Param(p) => {
                    for loc in points_to.get(func.name.as_str()).unwrap().get(p).unwrap() {
                        expanded_accesses
                            .entry(*loc)
                            .or_default()
                            .insert((*access_type, *root_id));
                    }
                }
            }
        }

        // println!("{}: {:#?}", func.name, expanded_accesses);

        for (loc_id, accesses) in expanded_accesses {
            if let Some((_, root_id)) = accesses
                .iter()
                .find(|(access_type, _)| matches!(access_type, AccessType::Write))
            {
                if accesses.iter().any(|it| it.1 != *root_id) {
                    eprintln!(
                        "possible aliased access to mem loc `{loc_id}` in `{}`",
                        func.name
                    );
                    return false;
                }
            }
        }
    }

    true
}

// TODO: Use visitor pattern to avoid duplicating code with analysis in harness?

fn visit_function<'a>(
    analysis: &mut Analysis<'a>,
    mut scope: Scope<'a>,
    func: &'a FnDecl,
) -> HashSet<(AccessType, RootIdentifier)> {
    let mut cx = FnContext {
        name: &func.name,
        accesses: Default::default(),
    };

    for (i, param) in func.inputs.iter().enumerate() {
        if let DataType::Ptr(_) = &param.data_type {
            scope
                .idents
                .insert(&param.name, RootIdentifier::Param(i as u32));
        }
    }

    for stmt in &func.body {
        visit_stmt(analysis, &mut scope, &mut cx, stmt);
    }

    cx.accesses
}

fn visit_stmt<'a>(
    analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    stmt: &'a Statement,
) {
    match stmt {
        Statement::LetDecl(stmt) => visit_expr(analysis, scope, cx, &stmt.initializer),
        Statement::VarDecl(stmt) => {
            if let Some(initializer) = &stmt.initializer {
                visit_expr(analysis, scope, cx, initializer);
            }

            scope
                .idents
                .insert(&stmt.ident, RootIdentifier::Mem(analysis.next_mem_loc()));
        }
        Statement::Assignment(stmt) => {
            visit_lhs(analysis, scope, cx, &stmt.lhs);
            visit_expr(analysis, scope, cx, &stmt.rhs);
        }
        Statement::Compound(block) => visit_stmt_block(analysis, scope, cx, block),
        Statement::If(stmt) => visit_if_stmt(analysis, scope, cx, stmt),
        Statement::Return(stmt) => {
            if let Some(value) = &stmt.value {
                visit_expr(analysis, scope, cx, value);
            }
        }
        Statement::Loop(stmt) => visit_stmt_block(analysis, scope, cx, &stmt.body),
        Statement::Break => {}
        Statement::Switch(stmt) => {
            visit_expr(analysis, scope, cx, &stmt.selector);

            for case in &stmt.cases {
                visit_stmt_block(analysis, scope, cx, &case.body);
            }

            visit_stmt_block(analysis, scope, cx, &stmt.default);
        }
        Statement::ForLoop(stmt) => {
            let mut scope = scope.clone();

            if let Some(init) = &stmt.header.init {
                match init {
                    ForLoopInit::VarDecl(stmt) => {
                        if let Some(init) = &stmt.initializer {
                            visit_expr(analysis, &mut scope, cx, init);
                        }

                        scope
                            .idents
                            .insert(&stmt.ident, RootIdentifier::Mem(analysis.next_mem_loc()));
                    }
                }
            }

            if let Some(condition) = &stmt.header.condition {
                visit_expr(analysis, &mut scope, cx, condition);
            }

            if let Some(update) = &stmt.header.update {
                match update {
                    ForLoopUpdate::Assignment(stmt) => {
                        visit_lhs(analysis, &mut scope, cx, &stmt.lhs);
                        visit_expr(analysis, &mut scope, cx, &stmt.rhs);
                    }
                }
            }

            visit_stmt_block(analysis, &mut scope, cx, &stmt.body);
        }
        Statement::FnCall(stmt) => {
            visit_function_call(analysis, scope, cx, &stmt.ident, &stmt.args);
        }
    }
}

fn visit_stmt_block<'a>(
    analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    block: &'a [Statement],
) {
    let mut scope = scope.clone();
    for stmt in block {
        visit_stmt(analysis, &mut scope, cx, stmt);
    }
}

fn visit_lhs<'a>(
    _analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    lhs: &'a AssignmentLhs,
) {
    if let AssignmentLhs::Expr(lhs) = &lhs {
        let ident = find_lhs_ident(lhs);
        let root_ident = scope.idents.get(ident);
        if let Some(root_ident) = root_ident {
            cx.accesses.insert((AccessType::Write, *root_ident));
        }
    }
}

fn visit_if_stmt<'a>(
    analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    stmt: &'a IfStatement,
) {
    visit_expr(analysis, scope, cx, &stmt.condition);
    visit_stmt_block(analysis, scope, cx, &stmt.body);

    if let Some(else_) = &stmt.else_ {
        match else_.as_ref() {
            Else::If(stmt) => visit_if_stmt(analysis, scope, cx, stmt),
            Else::Else(body) => visit_stmt_block(analysis, scope, cx, body),
        }
    }
}

fn visit_expr<'a>(
    analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    node: &'a ExprNode,
) {
    match &node.expr {
        Expr::Lit(_) => {}
        Expr::TypeCons(expr) => {
            for arg in &expr.args {
                visit_expr(analysis, scope, cx, arg);
            }
        }
        Expr::Var(expr) => {
            let ident = scope.idents.get(expr.ident.as_str());
            if let Some(root_ident) = ident {
                cx.accesses.insert((AccessType::Read, *root_ident));
            }
        }
        Expr::Postfix(expr) => {
            visit_expr(analysis, scope, cx, &expr.inner);

            match &expr.postfix {
                Postfix::Index(index) => visit_expr(analysis, scope, cx, index),
                Postfix::Member(_) => {}
            }
        }
        Expr::UnOp(expr) => {
            visit_expr(analysis, scope, cx, &expr.inner);
        }
        Expr::BinOp(expr) => {
            visit_expr(analysis, scope, cx, &expr.left);
            visit_expr(analysis, scope, cx, &expr.right);
        }
        Expr::FnCall(expr) => {
            visit_function_call(analysis, scope, cx, &expr.ident, &expr.args);
        }
    }
}

fn find_lhs_ident(node: &LhsExprNode) -> &str {
    match &node.expr {
        LhsExpr::Ident(v) => v,
        LhsExpr::Postfix(node, _) => find_lhs_ident(node),
        LhsExpr::Deref(node) => find_lhs_ident(node),
        LhsExpr::AddressOf(node) => find_lhs_ident(node),
    }
}

fn visit_function_call<'a>(
    analysis: &mut Analysis<'a>,
    scope: &mut Scope<'a>,
    cx: &mut FnContext<'a>,
    ident: &'a str,
    args: &'a [ExprNode],
) {
    let mut has_pointer_args = false;
    let mut arg_ids = vec![];

    for (i, arg) in args.iter().enumerate() {
        if let DataType::Ptr(_) = arg.data_type {
            has_pointer_args = true;
            let root_ident = scope.idents.get(find_pointer_expr_root(arg)).unwrap();
            match root_ident {
                RootIdentifier::Mem(loc) => {
                    analysis
                        .points_to
                        .entry(ident)
                        .or_default()
                        .entry(i as u32)
                        .or_default()
                        .insert(*loc);
                }
                RootIdentifier::Param(id) => {
                    let locs =
                        analysis.points_to.get(cx.name).unwrap().get(id).unwrap() as *const _;

                    analysis
                        .points_to
                        .entry(ident)
                        .or_default()
                        .entry(i as u32)
                        .or_default()
                        // This should be safe as long as cx.name != ident (which shouldn't be the
                        // case since recursion is forbidden in WGSL)
                        .extend(unsafe { &*locs });
                }
            }
            arg_ids.push(Some(*root_ident));
        } else {
            visit_expr(analysis, scope, cx, arg);
            arg_ids.push(None);
        }
    }

    if has_pointer_args {
        let id = analysis.next_id();
        analysis
            .calls
            .entry(cx.name)
            .or_default()
            .insert(FnCall(id, ident, arg_ids));
    }
}

fn find_pointer_expr_root(node: &ExprNode) -> &str {
    match &node.expr {
        Expr::Var(expr) => &expr.ident,
        Expr::Postfix(expr) => find_pointer_expr_root(&expr.inner),
        Expr::UnOp(expr) => find_pointer_expr_root(&expr.inner),
        _ => unreachable!("invalid subexpression encountered in pointer expression"),
    }
}
