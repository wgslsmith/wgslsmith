use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentOp, AssignmentStatement, BinOp, Else, Expr, ExprNode,
    FnAttr, FnDecl, FnInput, FnOutput, ForLoopHeader, ForLoopInit, ForLoopStatement, ForLoopUpdate,
    GlobalConstDecl, GlobalVarAttr, GlobalVarDecl, IfStatement, LetDeclStatement, Lit,
    LoopStatement, Module, Postfix, ReturnStatement, ShaderStage, Statement, StorageClass,
    StructDecl, StructMember, StructMemberAttr, SwitchCase, SwitchStatement, UnOp,
    VarDeclStatement, VarQualifier,
};
use indenter::Indented;
use peeking_take_while::PeekableExt;
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct WGSLParser;

#[derive(Clone)]
struct Environment {
    vars: HashMap<String, DataType>,
    fns: HashMap<u64, DataType>,
    types: HashMap<String, Rc<StructDecl>>,
}

impl Environment {
    pub fn new() -> Self {
        let built_in_fns = ast::gen_builtin_fns(["dot", "countOneBits", "reverseBits"].into_iter())
            .into_iter()
            .filter_map(|(name, params, ret_ty)| {
                let ret_ty = ret_ty?;
                let hash = Self::hash_fn(&name, params.iter());
                Some((hash, ret_ty))
            })
            .collect();

        Environment {
            vars: HashMap::new(),
            fns: built_in_fns,
            types: HashMap::new(),
        }
    }

    pub fn var(&self, name: &str) -> Option<&DataType> {
        self.vars.get(name)
    }

    pub fn insert_var(&mut self, name: String, ty: DataType) {
        self.vars.insert(name, ty);
    }

    pub fn ty(&self, name: &str) -> Option<&Rc<StructDecl>> {
        self.types.get(name)
    }

    pub fn insert_struct(&mut self, name: String, decl: Rc<StructDecl>) {
        self.types.insert(name, decl);
    }

    pub fn fun<'a, 'b>(
        &'b self,
        name: &str,
        params: impl Iterator<Item = &'a DataType>,
    ) -> Option<&'b DataType> {
        self.fns.get(&Self::hash_fn(name, params))
    }

    pub fn insert_fun<'a>(
        &mut self,
        name: &str,
        params: impl Iterator<Item = &'a DataType>,
        ret_ty: DataType,
    ) {
        self.fns.insert(Self::hash_fn(name, params), ret_ty);
    }

    fn hash_fn<'a>(name: &str, params: impl Iterator<Item = &'a DataType>) -> u64 {
        let mut hasher = DefaultHasher::new();

        hasher.write(name.as_bytes());

        for param in params {
            param.hash(&mut hasher);
        }

        hasher.finish()
    }
}

pub fn parse(input: &str) -> Module {
    let pairs = WGSLParser::parse(Rule::translation_unit, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_translation_unit(pair, &mut Environment::new())
}

pub fn parse_fn(input: &str) -> FnDecl {
    let pairs = WGSLParser::parse(Rule::function_decl, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_function_decl(pair, &mut Environment::new())
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
            let storage_class = match pairs.next().unwrap().as_str() {
                "function" => StorageClass::Function,
                "private" => StorageClass::Private,
                "workgroup" => StorageClass::WorkGroup,
                "uniform" => StorageClass::Uniform,
                "storage" => StorageClass::Storage,
                _ => unreachable!(),
            };

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
        expr = Some(match pair.as_rule() {
            Rule::literal_expression => parse_literal_expression(pair),
            Rule::ident => parse_var_expression(pair, &Environment::new()),
            _ => panic!("{:#?}", pair),
        })
    }

    let data_type = data_type.unwrap_or_else(|| {
        expr.as_ref()
            .expect("var declaration must have type or initializer")
            .data_type
            .clone()
    });

    env.insert_var(name.clone(), data_type.clone());

    GlobalVarDecl {
        attrs,
        qualifier,
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
    env.insert_fun(
        &decl.name,
        decl.members.iter().map(|it| &it.data_type),
        DataType::Struct(decl.clone()),
    );

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
                    "stage" => FnAttr::Stage(match pairs.next().unwrap().as_str() {
                        "compute" => ShaderStage::Compute,
                        "vertex" => ShaderStage::Vertex,
                        "fragment" => ShaderStage::Fragment,
                        _ => panic!("invalid argument for stage attr"),
                    }),
                    "workgroup_size" => FnAttr::WorkgroupSize(
                        match parse_literal_expression(pairs.next().unwrap()).expr {
                            Expr::Lit(Lit::Int(v)) => v.try_into().unwrap(),
                            Expr::Lit(Lit::UInt(v)) => v,
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
        env.insert_fun(
            &name,
            inputs.iter().map(|i| &i.data_type),
            output.data_type.clone(),
        );
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
        Rule::switch_statement => parse_switch_statement(pair, env),
        Rule::for_statement => parse_for_statement(pair, env),
        _ => unreachable!(),
    }
}

fn parse_let_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    let initializer = parse_expression(pairs.next().unwrap(), env);
    env.insert_var(ident.clone(), initializer.data_type.clone());
    LetDeclStatement::new(ident, initializer).into()
}

fn parse_var_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();

    let mut pair = pairs.next();

    let ty = if let Some(Rule::type_decl) = pair.as_ref().map(|it| it.as_rule()) {
        let ty = parse_type_decl(pair.unwrap(), env);
        pair = pairs.next();
        Some(ty)
    } else {
        None
    };

    let expression = if let Some(Rule::expression) = pair.as_ref().map(|it| it.as_rule()) {
        Some(parse_expression(pair.unwrap(), env))
    } else {
        None
    };

    env.insert_var(
        ident.clone(),
        ty.as_ref()
            .unwrap_or_else(|| &expression.as_ref().unwrap().data_type)
            .clone(),
    );

    VarDeclStatement::new(ident, ty, expression).into()
}

fn parse_assignment_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let lhs = parse_lhs_expression(pairs.next().unwrap(), env);
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
    ReturnStatement::new(expression).into()
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

fn parse_lhs_expression(pair: Pair<Rule>, env: &Environment) -> AssignmentLhs {
    if pair.as_str() == "_" {
        return AssignmentLhs::Underscore;
    }

    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();

    let postfixes = pairs
        .map(|pair| {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::expression => Postfix::ArrayIndex(Box::new(parse_expression(pair, env))),
                Rule::ident => Postfix::Member(pair.as_str().to_owned()),
                _ => unreachable!(),
            }
        })
        .collect();

    AssignmentLhs::Simple(ident, postfixes)
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
        let op: BinOp = op.as_rule().into();

        ExprNode {
            data_type: op.type_eval(&l.data_type, &r.data_type),
            expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
        }
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
        _ => unreachable!(),
    };

    let expr = parse_unary_expression(pairs.next().unwrap(), env);

    ExprNode {
        data_type: op.type_eval(&expr.data_type),
        expr: Expr::UnOp(op, Box::new(expr)),
    }
}

fn parse_singular_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();
    let mut expr = parse_primary_expression(pairs.next().unwrap(), env);

    for pf in pairs {
        let pair = pf.into_inner().next().unwrap();
        let pf = match pair.as_rule() {
            Rule::expression => Postfix::ArrayIndex(Box::new(parse_expression(pair, env))),
            Rule::ident => Postfix::Member(pair.as_str().to_owned()),
            _ => unreachable!(),
        };

        let data_type = match pf {
            Postfix::ArrayIndex(_) => match &expr.data_type {
                DataType::Scalar(_) => panic!("cannot index a scalar"),
                DataType::Vector(_, t) => DataType::Scalar(*t),
                DataType::Array(t, _) => (**t).clone(),
                DataType::Struct(_) => panic!("cannot index a struct"),
            },
            Postfix::Member(ref field) => match &expr.data_type {
                DataType::Scalar(_) => panic!("cannot access member of a scalar"),
                DataType::Vector(_, t) => {
                    if field.len() == 1 {
                        DataType::Scalar(*t)
                    } else {
                        DataType::Vector(field.len() as u8, *t)
                    }
                }
                DataType::Array(_, _) => panic!("cannot access member of an array"),
                // We need a type environment for this
                DataType::Struct(t) => t
                    .members
                    .iter()
                    .find(|m| m.name == *field)
                    .unwrap()
                    .data_type
                    .clone(),
            },
        };

        expr = ExprNode {
            data_type,
            expr: Expr::Postfix(Box::new(expr), pf),
        };
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
            Lit::UInt(pair.as_str().trim_end_matches('u').parse().unwrap()),
        ),
        Rule::int_literal => (ScalarType::I32, Lit::Int(pair.as_str().parse().unwrap())),
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

    ExprNode {
        data_type: t.clone(),
        expr: Expr::TypeCons(t, args),
    }
}

fn parse_call_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();

    let ident = pairs.next().unwrap();
    let args = pairs
        .map(|pair| parse_expression(pair, env))
        .collect::<Vec<_>>();

    struct FunSig<'a>(&'a str, &'a [ExprNode]);

    impl<'a> std::fmt::Display for FunSig<'a> {
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

    ExprNode {
        data_type: env
            .fun(ident.as_str(), args.iter().map(|arg| &arg.data_type))
            .unwrap_or_else(|| panic!("`{}` not found", FunSig(ident.as_str(), &args)))
            .clone(),
        expr: Expr::FnCall(ident.as_str().to_owned(), args),
    }
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
        Rule::ident => DataType::Struct(
            env.ty(pair.as_str())
                .unwrap_or_else(|| panic!("type not found: {}", pair.as_str()))
                .clone(),
        ),
        _ => panic!("{}", pair),
    }
}

fn parse_var_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    ExprNode {
        data_type: env
            .var(pair.as_str())
            .expect("variable must be defined before use")
            .clone(),
        expr: Expr::Var(pair.as_str().to_owned()),
    }
}

fn parse_paren_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    parse_expression(pair, env)
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
                const EXPECTED: &str = include_str!(concat!("tests/", stringify!($name), ".ast"));
                let pairs = WGSLParser::parse(Rule::translation_unit, SRC).unwrap();
                let pair = pairs.into_iter().next().unwrap();
                let module = parse_translation_unit(pair, &mut Environment::new());
                pretty_assertions::assert_eq!(
                    format!("{}", DebugModule(&module)).trim(),
                    EXPECTED.trim().replace("\r\n", "\n"),
                );
            }
        };
    }

    test_case!(structs);
    test_case!(loops);

    test_case!(test_1);
    test_case!(test_2);
    test_case!(test_3);
    test_case!(test_4);
    test_case!(test_5);
}

pub struct DebugModule<'a>(pub &'a Module);

fn indented<D>(f: &mut D) -> Indented<'_, D> {
    indenter::indented(f).with_str("  ")
}

impl Display for DebugModule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module")?;

        for decl in &self.0.structs {
            writeln!(indented(f), "{}", DebugStruct(decl))?;
        }

        for decl in &self.0.vars {
            writeln!(indented(f), "{}", DebugGlobalVar(decl))?;
        }

        for decl in &self.0.functions {
            writeln!(indented(f), "{}", DebugFn(decl))?;
        }

        Ok(())
    }
}

struct DebugStruct<'a>(&'a StructDecl);

impl Display for DebugStruct<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "struct `{}`", self.0.name)?;

        for member in &self.0.members {
            writeln!(f)?;
            write!(indented(f), "member `{}` {}", member.name, member.data_type)?;
        }

        Ok(())
    }
}

struct DebugGlobalVar<'a>(&'a GlobalVarDecl);

impl Display for DebugGlobalVar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var `{}` {}", self.0.name, self.0.data_type)?;

        for attr in &self.0.attrs {
            writeln!(f)?;
            write!(indented(f), "attr {:?}", attr)?;
        }

        if let Some(qualifier) = &self.0.qualifier {
            writeln!(f)?;
            write!(indented(f), "qualifier stcls:{}", qualifier.storage_class)?;

            if let Some(access_mode) = &qualifier.access_mode {
                write!(f, " mode:{access_mode}")?;
            }
        }

        if let Some(init) = &self.0.initializer {
            writeln!(f)?;
            write!(indented(f), "{}", DebugExpr(init))?;
        }

        Ok(())
    }
}

struct DebugFn<'a>(&'a FnDecl);

impl Display for DebugFn<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn `{}`", self.0.name)?;

        for attr in &self.0.attrs {
            writeln!(f)?;
            write!(indented(f), "attr {:?}", attr)?;
        }

        for stmt in &self.0.body {
            writeln!(f)?;
            write!(indented(f), "{}", DebugStmt(stmt))?;
        }

        Ok(())
    }
}

struct DebugStmt<'a>(&'a Statement);

impl Display for DebugStmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stmt ")?;

        match self.0 {
            Statement::LetDecl(LetDeclStatement { ident, initializer }) => {
                writeln!(f, "'let' `{ident}`")?;
                write!(indented(f), "{}", DebugExpr(initializer))?;
            }
            Statement::VarDecl(VarDeclStatement {
                ident, initializer, ..
            }) => {
                writeln!(f, "'var' `{ident}`")?;
                if let Some(init) = initializer {
                    write!(indented(f), "{}", DebugExpr(init))?;
                }
            }
            Statement::Assignment(AssignmentStatement { lhs, op, rhs }) => {
                writeln!(f, "'ass' `{op}`")?;
                write!(indented(f), "lhs ")?;

                match lhs {
                    AssignmentLhs::Underscore => write!(f, "_")?,
                    AssignmentLhs::Simple(id, pf) => {
                        write!(f, "`{id}`")?;

                        let f = &mut indented(f);
                        for pf in pf {
                            writeln!(f)?;
                            match pf {
                                Postfix::ArrayIndex(e) => {
                                    write!(indented(f), "array_index {}", DebugExpr(e))?
                                }
                                Postfix::Member(id) => write!(indented(f), "member `{id}`")?,
                            }
                        }
                    }
                }

                writeln!(f)?;
                write!(indented(f), "rhs {}", DebugExpr(rhs))?;
            }
            Statement::Compound(block) => {
                write!(f, "'block'")?;

                for stmt in block {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugStmt(stmt))?;
                }
            }
            Statement::If(IfStatement {
                condition, body, ..
            }) => {
                writeln!(f, "'if'")?;
                writeln!(indented(f), "{}", DebugExpr(condition))?;
                write!(indented(f), "body")?;

                let f = &mut indented(f);
                for stmt in body {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugStmt(stmt))?;
                }
            }
            Statement::Return(stmt) => {
                write!(f, "'return'")?;

                if let Some(v) = &stmt.value {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugExpr(v))?;
                }
            }
            Statement::Loop(stmt) => {
                write!(f, "'loop'")?;

                for stmt in &stmt.body {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugStmt(stmt))?;
                }
            }
            Statement::Break => write!(f, "'break'")?,
            Statement::Switch(_) => {
                todo!()
            }
            Statement::ForLoop(_) => todo!(),
        }

        Ok(())
    }
}

struct DebugExpr<'a>(&'a ExprNode);

impl Display for DebugExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expr ")?;

        match &self.0.expr {
            Expr::Lit(lit) => write!(f, "'lit' '{lit}' {}", self.0.data_type)?,
            Expr::TypeCons(ty, args) => {
                write!(f, "'cons' {} {}", ty, self.0.data_type)?;

                for arg in args {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugExpr(arg))?;
                }
            }
            Expr::Var(id) => write!(f, "`{id}` {}", self.0.data_type)?,
            Expr::Postfix(e, pf) => {
                writeln!(f, "'pf' {}", self.0.data_type)?;
                writeln!(indented(f), "{}", DebugExpr(e))?;

                match pf {
                    Postfix::ArrayIndex(e) => write!(indented(f), "array_index {}", DebugExpr(e))?,
                    Postfix::Member(id) => write!(indented(f), "member `{id}`")?,
                }
            }
            Expr::UnOp(op, e) => {
                writeln!(f, "'unop' '{op}' {}", self.0.data_type)?;
                write!(indented(f), "{}", DebugExpr(e))?;
            }
            Expr::BinOp(op, l, r) => {
                writeln!(f, "'binop' '{op}' {}", self.0.data_type)?;
                writeln!(indented(f), "{}", DebugExpr(l))?;
                write!(indented(f), "{}", DebugExpr(r))?;
            }
            Expr::FnCall(id, args) => {
                write!(f, "'fncall' `{id}` {}", self.0.data_type)?;

                for arg in args {
                    writeln!(f)?;
                    write!(indented(f), "{}", DebugExpr(arg))?;
                }
            }
        }

        Ok(())
    }
}
