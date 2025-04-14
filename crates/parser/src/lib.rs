use std::hash::Hash;
use std::rc::Rc;

use ast::types::{DataType, MemoryViewType, ScalarType};
use ast::*;
use peeking_take_while::PeekableExt;
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use rpds::HashTrieMap;
use strum::IntoEnumIterator;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct WGSLParser;

enum Func {
    Builtin(BuiltinFn),
    User(DataType),
}

impl Func {
    pub fn return_type<'a>(&self, params: impl Iterator<Item = &'a DataType>) -> Option<DataType> {
        match self {
            Func::Builtin(ty) => ty.return_type(params),
            Func::User(return_type) => Some(return_type.clone()),
        }
    }
}

#[derive(Clone, Default)]
pub struct Environment {
    vars: HashTrieMap<String, DataType>,
    fns: HashTrieMap<String, Func>,
    types: HashTrieMap<String, Rc<StructDecl>>,
}

fn builtins() -> HashTrieMap<String, Func> {
    BuiltinFn::iter()
        .map(|it| (it.as_ref().to_owned(), Func::Builtin(it)))
        .collect()
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            vars: HashTrieMap::new(),
            fns: builtins(),
            types: HashTrieMap::new(),
        }
    }

    pub fn var(&self, name: &str) -> Option<&DataType> {
        self.vars.get(name)
    }

    pub fn insert_var(&mut self, name: String, ty: DataType) {
        self.vars.insert_mut(name, ty);
    }

    pub fn ty(&self, name: &str) -> Option<&Rc<StructDecl>> {
        self.types.get(name)
    }

    pub fn insert_struct(&mut self, name: String, decl: Rc<StructDecl>) {
        self.types.insert_mut(name, decl);
    }

    pub fn func<'a>(
        &self,
        name: &str,
        params: impl Iterator<Item = &'a DataType>,
    ) -> Option<DataType> {
        self.fns.get(name).and_then(|it| it.return_type(params))
    }

    pub fn insert_func(&mut self, name: String, ret_ty: DataType) {
        self.fns.insert_mut(name, Func::User(ret_ty));
    }
}

pub fn parse(input: &str) -> Module {
    let pairs = WGSLParser::parse(Rule::translation_unit, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_translation_unit(pair, &mut Environment::new())
}

pub fn parse_fn(input: &str, env: &mut Environment) -> FnDecl {
    let pairs = WGSLParser::parse(Rule::function_decl, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_function_decl(pair, env)
}

fn parse_translation_unit(pair: Pair<Rule>, env: &mut Environment) -> Module {
    let decls = pair
        .into_inner()
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| parse_global_decl(pair, env))
        .collect::<Vec<_>>();

    let mut functions = vec![];
    let mut structs = vec![];
    let mut consts = vec![];
    let mut vars = vec![];

    for decl in decls {
        match decl {
            GlobalDecl::Const(decl) => consts.push(decl),
            GlobalDecl::Var(decl) => vars.push(decl),
            GlobalDecl::Struct(decl) => structs.push(decl),
            GlobalDecl::Fn(decl) => functions.push(decl),
        }
    }

    Module {
        functions,
        structs,
        consts,
        vars,
    }
}

enum GlobalDecl {
    Const(GlobalConstDecl),
    Var(GlobalVarDecl),
    Struct(Rc<StructDecl>),
    Fn(FnDecl),
}

fn parse_global_decl(pair: Pair<Rule>, env: &mut Environment) -> GlobalDecl {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::global_constant_decl => GlobalDecl::Const(parse_global_const_decl(pair, env)),
        Rule::global_variable_decl => GlobalDecl::Var(parse_global_variable_decl(pair, env)),
        Rule::struct_decl => GlobalDecl::Struct(parse_struct_decl(pair, env)),
        Rule::function_decl => GlobalDecl::Fn(parse_function_decl(pair, env)),
        _ => unreachable!(),
    }
}

fn parse_global_const_decl(pair: Pair<Rule>, env: &mut Environment) -> GlobalConstDecl {
    let mut pairs = pair.into_inner().peekable();

    let name = pairs.next().unwrap().as_str().to_owned();
    let mut data_type = None;

    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::type_decl {
            let pair = pairs.next().unwrap();
            data_type = Some(parse_type_decl(pair, env));
        }
    }

    let expr = parse_expression(pairs.next().unwrap(), env);
    let data_type = data_type.unwrap_or_else(|| expr.data_type.clone());

    env.insert_var(name.clone(), data_type.clone());

    GlobalConstDecl {
        name,
        data_type,
        initializer: expr,
    }
}

fn parse_global_variable_decl(pair: Pair<Rule>, env: &mut Environment) -> GlobalVarDecl {
    let mut pairs = pair.into_inner().peekable();

    let attrs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::attribute_list)
        .flat_map(|pair| {
            pair.into_inner().map(|pair| {
                let mut pairs = pair.into_inner();
                let name = pairs.next().unwrap().as_str();
                let arg = pairs.next().unwrap().as_str();
                match name {
                    "binding" => GlobalVarAttr::Binding(arg.parse().unwrap()),
                    "group" => GlobalVarAttr::Group(arg.parse().unwrap()),
                    _ => panic!("invalid global variable attribute: {}", name),
                }
            })
        })
        .collect();

    let mut qualifier = None;

    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::variable_qualifier {
            let mut pairs = pairs.next().unwrap().into_inner();
            let storage_class = parse_storage_class(pairs.next().unwrap());

            let access_mode = if matches!(pairs.peek(), Some(access_mode) if access_mode.as_rule() == Rule::access_mode)
            {
                Some(match pairs.next().unwrap().as_str() {
                    "read" => AccessMode::Read,
                    "write" => AccessMode::Write,
                    "read_write" => AccessMode::ReadWrite,
                    _ => unreachable!(),
                })
            } else {
                None
            };

            qualifier = Some(VarQualifier {
                storage_class,
                access_mode,
            })
        }
    }

    let qualifier = qualifier.expect("module scope var declaration must specify storage class");
    let name = pairs.next().unwrap().as_str().to_owned();
    let mut data_type = None;
    let mut expr = None;

    if let Some(pair) = pairs.peek() {
        if pair.as_rule() == Rule::type_decl {
            let pair = pairs.next().unwrap();
            data_type = Some(parse_type_decl(pair, env));
        }
    }

    if pairs.peek().is_some() {
        let pair = pairs.next().unwrap();
        expr = Some(parse_expression(pair, env))
    }

    let data_type = data_type.unwrap_or_else(|| {
        expr.as_ref()
            .expect("var declaration must have type or initializer")
            .data_type
            .clone()
    });

    let mut ref_view = MemoryViewType::new(data_type.clone(), qualifier.storage_class);
    if let Some(access_mode) = qualifier.access_mode {
        ref_view.access_mode = access_mode;
    }

    env.insert_var(name.clone(), DataType::Ref(ref_view));

    GlobalVarDecl {
        attrs,
        qualifier: Some(qualifier),
        name,
        data_type,
        initializer: expr,
    }
}

fn parse_struct_decl(pair: Pair<Rule>, env: &mut Environment) -> Rc<StructDecl> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_owned();
    let members = pairs
        .map(|pair| {
            let mut pairs = pair.into_inner().peekable();

            let attrs = pairs
                .by_ref()
                .peeking_take_while(|pair| pair.as_rule() == Rule::attribute_list)
                .flat_map(|pair| {
                    pair.into_inner().map(|pair| {
                        let mut pairs = pair.into_inner();
                        let name = pairs.next().unwrap().as_str();
                        let arg = pairs.next().unwrap().as_str();
                        match name {
                            "align" => StructMemberAttr::Align(arg.parse().unwrap()),
                            _ => panic!("invalid struct member attribute: {}", name),
                        }
                    })
                })
                .collect();

            let name = pairs.next().unwrap().as_str().to_owned();
            let data_type = parse_type_decl(pairs.next().unwrap(), env);
            StructMember::new(attrs, name, data_type)
        })
        .collect();

    let decl = StructDecl::new(name.clone(), members);

    env.insert_struct(name, decl.clone());
    env.insert_func(decl.name.clone(), DataType::Struct(decl.clone()));

    decl
}

fn parse_function_decl(pair: Pair<Rule>, env: &mut Environment) -> FnDecl {
    let mut pairs = pair.into_inner().peekable();

    let attrs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::attribute_list)
        .flat_map(|pair| {
            pair.into_inner().map(|pair| {
                let mut pairs = pair.into_inner();
                let name = pairs.next().unwrap().as_str();
                match name {
                    "compute" => FnAttr::Stage(ShaderStage::Compute),
                    "vertex" => FnAttr::Stage(ShaderStage::Vertex),
                    "fragment" => FnAttr::Stage(ShaderStage::Fragment),
                    "stage" => FnAttr::Stage(match pairs.next().unwrap().as_str() {
                        "compute" => ShaderStage::Compute,
                        "vertex" => ShaderStage::Vertex,
                        "fragment" => ShaderStage::Fragment,
                        _ => panic!("invalid argument for stage attr"),
                    }),
                    "workgroup_size" => FnAttr::WorkgroupSize(
                        match parse_literal_expression(pairs.next().unwrap()).expr {
                            Expr::Lit(Lit::I32(v)) => v.try_into().unwrap(),
                            Expr::Lit(Lit::U32(v)) => v,
                            _ => panic!("invalid argument for workgroup_size attr"),
                        },
                    ),
                    _ => panic!("invalid function attribute: {}", name),
                }
            })
        })
        .collect();

    let name = pairs.next().unwrap().as_str().to_owned();
    let inputs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::param)
        .map(|pair| {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str().to_owned();
            let data_type = parse_type_decl(pairs.next().unwrap(), env);
            FnInput {
                attrs: vec![],
                name,
                data_type,
            }
        })
        .collect::<Vec<_>>();

    let output = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::type_decl)
        .map(|pair| FnOutput {
            attrs: vec![],
            data_type: parse_type_decl(pair, env),
        })
        .next();

    if let Some(output) = &output {
        env.insert_func(name.clone(), output.data_type.clone());
    }

    let mut env = env.clone();
    for param in &inputs {
        env.insert_var(param.name.clone(), param.data_type.clone());
    }

    let body = parse_compound_statement(pairs.next().unwrap(), &env).into_compount_statement();

    FnDecl {
        attrs,
        name,
        inputs,
        output,
        body,
    }
}

fn parse_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let pair = if pair.as_rule() == Rule::statement {
        pair.into_inner().next().unwrap()
    } else {
        pair
    };

    match pair.as_rule() {
        Rule::let_statement => parse_let_statement(pair, env),
        Rule::var_statement => parse_var_statement(pair, env),
        Rule::assignment_statement => parse_assignment_statement(pair, env),
        Rule::compound_statement => parse_compound_statement(pair, env),
        Rule::if_statement => parse_if_statement(pair, env),
        Rule::return_statement => parse_return_statement(pair, env),
        Rule::loop_statement => parse_loop_statement(pair, env),
        Rule::break_statement => Statement::Break,
        Rule::continue_statement => Statement::Continue,
        Rule::fallthrough_statement => Statement::Fallthrough,
        Rule::switch_statement => parse_switch_statement(pair, env),
        Rule::for_statement => parse_for_statement(pair, env),
        Rule::call_statement => parse_call_statement(pair, env),
        _ => unreachable!(),
    }
}

fn parse_let_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    let initializer = parse_expression(pairs.next().unwrap(), env);
    let stmt = LetDeclStatement::new(ident.clone(), initializer);
    env.insert_var(ident, stmt.inferred_type().clone());
    stmt.into()
}

fn parse_var_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();

    let mut pair = pairs.next();

    let specified_type = if let Some(Rule::type_decl) = pair.as_ref().map(|it| it.as_rule()) {
        let ty = parse_type_decl(pair.unwrap(), env);
        pair = pairs.next();
        Some(ty)
    } else {
        None
    };

    let initializer = if let Some(Rule::expression) = pair.as_ref().map(|it| it.as_rule()) {
        Some(parse_expression(pair.unwrap(), env))
    } else {
        None
    };

    let stmt = VarDeclStatement::new(ident.clone(), specified_type, initializer);

    let ref_view = MemoryViewType::new(stmt.inferred_type().clone(), StorageClass::Function);
    env.insert_var(ident, DataType::Ref(ref_view));

    stmt.into()
}

fn parse_assignment_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();

    let lhs = parse_assignment_lhs(pairs.next().unwrap(), env);
    let op = pairs.next().unwrap();
    let rhs = parse_expression(pairs.next().unwrap(), env);

    let op = op.into_inner().next().unwrap();
    let op = match op.as_rule() {
        Rule::op_assign => AssignmentOp::Simple,
        Rule::compound_assignment_operator => match op.into_inner().next().unwrap().as_rule() {
            Rule::op_plus_equal => AssignmentOp::Plus,
            Rule::op_minus_equal => AssignmentOp::Minus,
            Rule::op_times_equal => AssignmentOp::Times,
            Rule::op_divide_equal => AssignmentOp::Divide,
            Rule::op_mod_equal => AssignmentOp::Mod,
            Rule::op_and_equal => AssignmentOp::And,
            Rule::op_or_equal => AssignmentOp::Or,
            Rule::op_xor_equal => AssignmentOp::Xor,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    AssignmentStatement::new(lhs, op, rhs).into()
}

fn parse_assignment_lhs(pair: Pair<Rule>, env: &Environment) -> AssignmentLhs {
    match pair.as_rule() {
        Rule::lhs_phony => AssignmentLhs::Phony,
        Rule::lhs_expression => AssignmentLhs::Expr(parse_lhs_expression(pair, env)),
        _ => unreachable!(),
    }
}

fn parse_compound_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut inner_env = env.clone();
    Statement::Compound(
        pair.into_inner()
            .map(|pair| parse_statement(pair, &mut inner_env))
            .collect(),
    )
}

fn parse_if_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let condition = parse_paren_expression(pairs.next().unwrap(), env);
    let block = parse_compound_statement(pairs.next().unwrap(), env).into_compount_statement();

    let els = pairs
        .next()
        .map(|pair| match pair.as_rule() {
            Rule::compound_statement => parse_compound_statement(pair, env),
            Rule::if_statement => parse_if_statement(pair, env),
            _ => unreachable!(),
        })
        .map(|stmt| match stmt {
            Statement::Compound(stmts) => Else::Else(stmts),
            Statement::If(stmt) => Else::If(stmt),
            _ => unreachable!(),
        });

    IfStatement::new(condition, block).with_else(els).into()
}

fn parse_return_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let expression = pair
        .into_inner()
        .next()
        .map(|pair| parse_expression(pair, env));

    if let Some(value) = expression {
        ReturnStatement::new(value).into()
    } else {
        ReturnStatement::none().into()
    }
}

fn parse_loop_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let block = parse_compound_statement(pairs.next().unwrap(), env).into_compount_statement();
    LoopStatement::new(block).into()
}

fn parse_switch_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();

    let expr = parse_expression(pairs.next().unwrap(), env);

    let mut cases = vec![];
    let mut default = None;

    for pair in pairs {
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();

        if pair.as_rule() == Rule::expression {
            let selector = parse_expression(pair, env);
            let body =
                parse_compound_statement(pairs.next().unwrap(), env).into_compount_statement();
            cases.push(SwitchCase { selector, body });
        } else {
            default = Some(parse_compound_statement(pair, env).into_compount_statement());
        }
    }

    let default = default.expect("switch statement must have default case");

    SwitchStatement::new(expr, cases, default).into()
}

fn parse_for_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();

    let mut pair = pairs.next().unwrap();

    let mut init = None;
    if pair.as_rule() == Rule::for_init {
        match parse_statement(pair.into_inner().next().unwrap(), env) {
            Statement::VarDecl(stmt) => {
                init = Some(ForLoopInit::VarDecl(stmt));
            }
            _ => panic!("only assignment statement is currently supported in for loop init"),
        };
        pair = pairs.next().unwrap();
    }

    let mut condition = None;
    if pair.as_rule() == Rule::expression {
        condition = Some(parse_expression(pair, env));
        pair = pairs.next().unwrap();
    }

    let mut update = None;
    if pair.as_rule() == Rule::for_update {
        match parse_statement(pair.into_inner().next().unwrap(), env) {
            Statement::Assignment(stmt) => {
                update = Some(ForLoopUpdate::Assignment(stmt));
            }
            _ => panic!("only assignment statement is currently supported in for loop init"),
        };
        pair = pairs.next().unwrap();
    }

    let body = parse_compound_statement(pair, env);

    let header = ForLoopHeader {
        init,
        condition,
        update,
    };

    ForLoopStatement::new(header, body.into_compount_statement()).into()
}

fn parse_call_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let pair = pair.into_inner().next().unwrap();
    let mut pairs = pair.into_inner();

    let ident = pairs.next().unwrap().as_str().to_owned();
    let args = pairs.map(|it| parse_expression(it, env)).collect();

    FnCallStatement::new(ident, args).into()
}

fn parse_lhs_expression(pair: Pair<Rule>, env: &Environment) -> LhsExprNode {
    let mut pairs = pair.into_inner().peekable();

    let prefixes: Vec<_> = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() != Rule::core_lhs_expression)
        .collect();

    fn parse_core(pair: Pair<Rule>, env: &Environment) -> LhsExprNode {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::lhs_expression => parse_lhs_expression(pair, env),
            Rule::ident => {
                let ident = pair.as_str().to_owned();
                LhsExprNode {
                    data_type: env
                        .var(&ident)
                        .expect("variable must be defined before use")
                        .clone(),
                    expr: LhsExpr::Ident(ident),
                }
            }
            _ => unreachable!(),
        }
    }

    let node = parse_core(pairs.next().unwrap(), env);
    let node = pairs.fold(node, |node, pair| {
        let pair = pair.into_inner().next().unwrap();
        let postfix = match pair.as_rule() {
            Rule::expression => Postfix::Index(Box::new(parse_expression(pair, env))),
            Rule::ident => Postfix::Member(pair.as_str().to_owned()),
            _ => unreachable!(),
        };

        LhsExprNode {
            data_type: postfix.type_eval(&node.data_type),
            expr: LhsExpr::Postfix(Box::new(node), postfix),
        }
    });

    prefixes.iter().rev().fold(node, |node, pair| {
        let (data_type, expr) = match pair.as_rule() {
            Rule::op_address_of => (
                UnOp::AddressOf.type_eval(&node.data_type),
                LhsExpr::AddressOf(Box::new(node)),
            ),
            Rule::op_indirection => (
                UnOp::Deref.type_eval(&node.data_type),
                LhsExpr::Deref(Box::new(node)),
            ),
            _ => unreachable!(),
        };

        LhsExprNode { data_type, expr }
    })
}

fn precedence_table() -> PrecClimber<Rule> {
    PrecClimber::new(vec![
        // Level 1: bitwise operators
        Operator::new(Rule::op_bit_and, Assoc::Left)
            | Operator::new(Rule::op_bit_or, Assoc::Left)
            | Operator::new(Rule::op_bit_xor, Assoc::Left),
        // Level 2: short-circuiting or operator
        Operator::new(Rule::op_log_or, Assoc::Left),
        // Level 3: short-circuiting and operator
        Operator::new(Rule::op_log_and, Assoc::Left),
        // Level 4: comparison operators
        Operator::new(Rule::op_less, Assoc::Left)
            | Operator::new(Rule::op_less_eq, Assoc::Left)
            | Operator::new(Rule::op_greater, Assoc::Left)
            | Operator::new(Rule::op_greater_eq, Assoc::Left)
            | Operator::new(Rule::op_equal, Assoc::Left)
            | Operator::new(Rule::op_nequal, Assoc::Left),
        // Level 5: shift operators
        Operator::new(Rule::op_lshift, Assoc::Left) | Operator::new(Rule::op_rshift, Assoc::Left),
        // Level 6: additive operators
        Operator::new(Rule::op_plus, Assoc::Left) | Operator::new(Rule::op_minus, Assoc::Left),
        // Level 7: multiplicative operators
        Operator::new(Rule::op_times, Assoc::Left)
            | Operator::new(Rule::op_divide, Assoc::Left)
            | Operator::new(Rule::op_mod, Assoc::Left),
    ])
}

fn parse_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::infix_expression => parse_infix_expression(pair, env),
        Rule::unary_expression => parse_unary_expression(pair, env),
        _ => unreachable!(),
    }
}

fn parse_infix_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pairs = pair.into_inner();

    let unary = |pair| parse_unary_expression(pair, env);
    let infix = |l: ExprNode, op: Pair<Rule>, r: ExprNode| -> ExprNode {
        BinOpExpr::new(op.as_rule().into(), l, r).into()
    };

    precedence_table().climb(pairs, unary, infix)
}

fn parse_unary_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();

    let first_pair = pairs.next().unwrap();
    let op = match first_pair.as_rule() {
        Rule::singular_expression => return parse_singular_expression(first_pair, env),
        _ => first_pair,
    };

    let op = match op.as_rule() {
        Rule::op_minus => UnOp::Neg,
        Rule::op_log_not => UnOp::Not,
        Rule::op_bit_not => UnOp::BitNot,
        Rule::op_address_of => UnOp::AddressOf,
        Rule::op_indirection => UnOp::Deref,
        _ => unreachable!(),
    };

    let expr = parse_unary_expression(pairs.next().unwrap(), env);

    UnOpExpr::new(op, expr).into()
}

fn parse_singular_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();
    let mut expr = parse_primary_expression(pairs.next().unwrap(), env);

    for pf in pairs {
        let pair = pf.into_inner().next().unwrap();
        let pf = match pair.as_rule() {
            Rule::expression => Postfix::Index(Box::new(parse_expression(pair, env))),
            Rule::ident => Postfix::Member(pair.as_str().to_owned()),
            _ => unreachable!(),
        };

        expr = PostfixExpr::new(expr, pf).into();
    }

    expr
}

fn parse_primary_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::literal_expression => parse_literal_expression(pair),
        Rule::type_cons_expression => parse_type_cons_expression(pair, env),
        Rule::call_expression => parse_call_expression(pair, env),
        Rule::var_expression => parse_var_expression(pair, env),
        Rule::paren_expression => parse_paren_expression(pair, env),
        Rule::unary_expression => parse_unary_expression(pair, env),
        _ => unreachable!(),
    }
}

fn parse_literal_expression(pair: Pair<Rule>) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    let (t, lit) = match pair.as_rule() {
        Rule::bool_literal => (ScalarType::Bool, Lit::Bool(pair.as_str().parse().unwrap())),
        Rule::uint_literal => (
            ScalarType::U32,
            Lit::U32(pair.as_str().trim_end_matches('u').parse().unwrap()),
        ),
        Rule::int_literal => (
            ScalarType::I32,
            Lit::I32(if pair.as_str().chars().last().unwrap() != ')' {
                pair.as_str().trim_end_matches('i').parse().unwrap()
            } else {
                pair.as_str()
                    .trim_start_matches("i32(")
                    .trim_end_matches(')')
                    .parse()
                    .unwrap()
            }),
        ),
        Rule::float_literal => (
            ScalarType::F32,
            Lit::F32(pair.as_str().trim_end_matches('f').parse().unwrap()),
        ),
        _ => unreachable!(),
    };

    ExprNode {
        data_type: DataType::Scalar(t),
        expr: Expr::Lit(lit),
    }
}

fn parse_type_cons_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();
    let t_decl = pairs.next().unwrap();

    let t = parse_type_decl(t_decl, env);
    let args = pairs.map(|pair| parse_expression(pair, env)).collect();

    TypeConsExpr::new(t, args).into()
}

fn parse_call_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();

    let ident = pairs.next().unwrap();
    let args = pairs
        .map(|pair| parse_expression(pair, env))
        .collect::<Vec<_>>();

    struct FunSig<'a>(&'a str, &'a [ExprNode]);

    impl std::fmt::Display for FunSig<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let FunSig(ident, args) = self;
            let mut args = args.iter();
            write!(f, "{ident}(")?;
            if let Some(arg) = args.next() {
                write!(f, "{}", arg.data_type)?;
            }
            for arg in args {
                write!(f, ", {}", arg.data_type)?;
            }
            write!(f, ")")
        }
    }

    let return_type = env
        .func(ident.as_str(), args.iter().map(|arg| &arg.data_type))
        .unwrap_or_else(|| panic!("`{}` not found", FunSig(ident.as_str(), &args)));

    FnCallExpr::new(ident.as_str().to_owned(), args).into_node(return_type)
}

fn parse_type_decl(pair: Pair<Rule>, env: &Environment) -> DataType {
    let pair = pair.into_inner().next().unwrap();

    fn parse_t_scalar(pair: Pair<Rule>) -> ScalarType {
        pair.into_inner().next().unwrap().as_rule().into()
    }

    match pair.as_rule() {
        Rule::t_scalar => DataType::Scalar(parse_t_scalar(pair)),
        Rule::t_vector => {
            let t_vector = pair.into_inner().next().unwrap();
            let n = match t_vector.as_rule() {
                Rule::t_vec2 => 2,
                Rule::t_vec3 => 3,
                Rule::t_vec4 => 4,
                _ => unreachable!(),
            };

            DataType::Vector(n, parse_t_scalar(t_vector.into_inner().next().unwrap()))
        }
        Rule::array_type_decl => {
            let mut pairs = pair.into_inner();
            let pair = pairs.next().unwrap();
            DataType::Array(
                Rc::new(parse_type_decl(pair, env)),
                pairs.next().map(|it| it.as_str().parse().unwrap()),
            )
        }
        Rule::ptr_type_decl => {
            let mut pairs = pair.into_inner();
            let storage_class = parse_storage_class(pairs.next().unwrap());
            let inner = parse_type_decl(pairs.next().unwrap(), env);
            DataType::Ptr(MemoryViewType::new(inner, storage_class))
        }
        Rule::ident => DataType::Struct(
            env.ty(pair.as_str())
                .unwrap_or_else(|| panic!("type not found: {}", pair.as_str()))
                .clone(),
        ),
        _ => panic!("{}", pair),
    }
}

fn parse_var_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    VarExpr::new(pair.as_str()).into_node(
        env.var(pair.as_str())
            .unwrap_or_else(|| panic!("variable `{}` must be defined before use", pair.as_str()))
            .clone(),
    )
}

fn parse_paren_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    parse_expression(pair, env)
}

fn parse_storage_class(pair: Pair<Rule>) -> StorageClass {
    match pair.as_str() {
        "function" => StorageClass::Function,
        "private" => StorageClass::Private,
        "workgroup" => StorageClass::WorkGroup,
        "uniform" => StorageClass::Uniform,
        "storage" => StorageClass::Storage,
        _ => unreachable!(),
    }
}

impl From<Rule> for BinOp {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::op_plus => BinOp::Plus,
            Rule::op_minus => BinOp::Minus,
            Rule::op_times => BinOp::Times,
            Rule::op_divide => BinOp::Divide,
            Rule::op_mod => BinOp::Mod,
            Rule::op_log_and => BinOp::LogAnd,
            Rule::op_log_or => BinOp::LogOr,
            Rule::op_bit_and => BinOp::BitAnd,
            Rule::op_bit_or => BinOp::BitOr,
            Rule::op_bit_xor => BinOp::BitXOr,
            Rule::op_lshift => BinOp::LShift,
            Rule::op_rshift => BinOp::RShift,
            Rule::op_equal => BinOp::Equal,
            Rule::op_nequal => BinOp::NotEqual,
            Rule::op_less => BinOp::Less,
            Rule::op_less_eq => BinOp::LessEqual,
            Rule::op_greater => BinOp::Greater,
            Rule::op_greater_eq => BinOp::GreaterEqual,
            _ => unreachable!(),
        }
    }
}

impl From<Rule> for ScalarType {
    fn from(rule: Rule) -> Self {
        match rule {
            Rule::t_bool => ScalarType::Bool,
            Rule::t_i32 => ScalarType::I32,
            Rule::t_u32 => ScalarType::U32,
            Rule::t_f32 => ScalarType::F32,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    macro_rules! test_case {
        ($name:ident) => {
            test_case!($name, $name);
        };
        ($name:ident, $fn:ident) => {
            #[test]
            fn $fn() {
                const SRC: &str = include_str!(concat!("tests/", stringify!($name), ".wgsl"));
                let pairs = WGSLParser::parse(Rule::translation_unit, SRC).unwrap();
                let pair = pairs.into_iter().next().unwrap();
                let module = parse_translation_unit(pair, &mut Environment::new());
                insta::assert_debug_snapshot!(module);
            }
        };
    }

    test_case!(calls);
    test_case!(floats);
    test_case!(loops);
    test_case!(ptrs);
    test_case!(structs);

    test_case!(test_1);
    test_case!(test_2);
    test_case!(test_3);
    test_case!(test_4);
    test_case!(test_5);
}
