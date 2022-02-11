use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, AttrList, BinOp, Expr, ExprNode, FnAttr, FnDecl, FnInput, FnOutput,
    GlobalVarAttr, GlobalVarDecl, Lit, Module, Postfix, ShaderStage, Statement, StorageClass,
    StructDecl, StructMember, UnOp, VarQualifier,
};
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
    types: HashMap<String, StructDecl>,
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

    pub fn ty(&self, name: &str) -> Option<&StructDecl> {
        self.types.get(name)
    }

    pub fn insert_struct(&mut self, name: String, decl: StructDecl) {
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

    let mut entrypoint = None;
    let mut functions = vec![];
    let mut structs = vec![];
    let mut vars = vec![];

    for decl in decls {
        match decl {
            GlobalDecl::Var(decl) => vars.push(decl),
            GlobalDecl::Struct(decl) => structs.push(decl),
            GlobalDecl::Fn(decl) => {
                if decl.name == "main" {
                    entrypoint = Some(decl)
                } else {
                    functions.push(decl)
                }
            }
        }
    }

    Module {
        entrypoint: entrypoint.expect("program must have an entrypoint"),
        functions,
        structs,
        vars,
    }
}

enum GlobalDecl {
    Var(GlobalVarDecl),
    Struct(StructDecl),
    Fn(FnDecl),
}

fn parse_global_decl(pair: Pair<Rule>, env: &mut Environment) -> GlobalDecl {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::global_variable_decl => GlobalDecl::Var(parse_global_variable_decl(pair, env)),
        Rule::struct_decl => GlobalDecl::Struct(parse_struct_decl(pair, env)),
        Rule::function_decl => GlobalDecl::Fn(parse_function_decl(pair, env)),
        _ => unreachable!(),
    }
}

fn parse_global_variable_decl(pair: Pair<Rule>, env: &mut Environment) -> GlobalVarDecl {
    let mut pairs = pair.into_inner().peekable();

    let attrs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::attribute_list)
        .map(|pair| {
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
        .flatten()
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
            data_type = Some(parse_type_decl(pair));
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

fn parse_struct_decl(pair: Pair<Rule>, env: &mut Environment) -> StructDecl {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap().as_str().to_owned();
    let members = pairs
        .map(|pair| {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str().to_owned();
            let data_type = parse_type_decl(pairs.next().unwrap());
            StructMember { name, data_type }
        })
        .collect();

    let decl = StructDecl {
        name: name.clone(),
        members,
    };

    env.insert_struct(name, decl.clone());

    decl
}

fn parse_function_decl(pair: Pair<Rule>, env: &mut Environment) -> FnDecl {
    let mut pairs = pair.into_inner().peekable();

    let attrs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::attribute_list)
        .map(|pair| {
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
        .flatten()
        .collect();

    let name = pairs.next().unwrap().as_str().to_owned();
    let inputs = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::param)
        .map(|pair| {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str().to_owned();
            let data_type = parse_type_decl(pairs.next().unwrap());
            FnInput {
                attrs: AttrList(vec![]),
                name,
                data_type,
            }
        })
        .collect::<Vec<_>>();

    let output = pairs
        .by_ref()
        .peeking_take_while(|pair| pair.as_rule() == Rule::type_decl)
        .map(|pair| FnOutput {
            attrs: AttrList(vec![]),
            data_type: parse_type_decl(pair),
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
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::let_statement => parse_let_statement(pair, env),
        Rule::var_statement => parse_var_statement(pair, env),
        Rule::assignment_statement => parse_assignment_statement(pair, env),
        Rule::compound_statement => parse_compound_statement(pair, env),
        Rule::if_statement => parse_if_statement(pair, env),
        Rule::return_statement => parse_return_statement(pair, env),
        Rule::loop_statement => parse_loop_statement(pair, env),
        Rule::break_statement => Statement::Break,
        _ => unreachable!(),
    }
}

fn parse_let_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    let expression = parse_expression(pairs.next().unwrap(), env);
    env.insert_var(ident.clone(), expression.data_type.clone());
    Statement::LetDecl(ident, expression)
}

fn parse_var_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    // TODO: Rhs for var is optional in grammar but not in the AST
    let expression = parse_expression(pairs.next().unwrap(), env);
    env.insert_var(ident.clone(), expression.data_type.clone());
    Statement::VarDecl(ident, expression)
}

fn parse_assignment_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let lhs = parse_lhs_expression(pairs.next().unwrap(), env);
    let rhs = parse_expression(pairs.next().unwrap(), env);
    Statement::Assignment(lhs, rhs)
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

    if pairs.next().is_some() {
        panic!("else and else-if is not currently support");
    }

    Statement::If(condition, block)
}

fn parse_return_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let expression = pair
        .into_inner()
        .next()
        .map(|pair| parse_expression(pair, env));
    Statement::Return(expression)
}

fn parse_loop_statement(pair: Pair<Rule>, env: &Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let block = parse_compound_statement(pairs.next().unwrap(), env).into_compount_statement();
    Statement::Loop(block)
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
        // Level 1: logical and bitwise operators
        Operator::new(Rule::op_log_and, Assoc::Left)
            | Operator::new(Rule::op_log_or, Assoc::Left)
            | Operator::new(Rule::op_bit_and, Assoc::Left)
            | Operator::new(Rule::op_bit_or, Assoc::Left)
            | Operator::new(Rule::op_bit_xor, Assoc::Left),
        // Level 2: comparison operators
        Operator::new(Rule::op_less, Assoc::Left)
            | Operator::new(Rule::op_less_eq, Assoc::Left)
            | Operator::new(Rule::op_greater, Assoc::Left)
            | Operator::new(Rule::op_greater_eq, Assoc::Left)
            | Operator::new(Rule::op_equal, Assoc::Left)
            | Operator::new(Rule::op_nequal, Assoc::Left),
        // Level 3: shift operators
        Operator::new(Rule::op_lshift, Assoc::Left) | Operator::new(Rule::op_rshift, Assoc::Left),
        // Level 4: additive operators
        Operator::new(Rule::op_plus, Assoc::Left) | Operator::new(Rule::op_minus, Assoc::Left),
        // Level 5: multiplicative operators
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
                DataType::Array(t) => (**t).clone(),
                DataType::User(_) => panic!("cannot index a struct"),
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
                DataType::Array(_) => panic!("cannot access member of an array"),
                // We need a type environment for this
                DataType::User(t) => env
                    .ty(t)
                    .unwrap_or_else(|| panic!("type not found: {}", t))
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

    let t = parse_type_decl(t_decl);
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

    ExprNode {
        data_type: env
            .fun(ident.as_str(), args.iter().map(|arg| &arg.data_type))
            .unwrap()
            .clone(),
        expr: Expr::FnCall(ident.as_str().to_owned(), args),
    }
}

fn parse_type_decl(pair: Pair<Rule>) -> DataType {
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
            let pair = pair.into_inner().next().unwrap();
            DataType::Array(Rc::new(parse_type_decl(pair)))
        }
        Rule::ident => DataType::User(Rc::new(pair.as_str().to_owned())),
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

    macro_rules! test_file {
        ($name:ident, $path:literal) => {
            #[test]
            fn $name() {
                const SRC: &str = include_str!($path);
                let pairs = WGSLParser::parse(Rule::translation_unit, SRC).unwrap();
                let pair = pairs.into_iter().next().unwrap();
                let module = parse_translation_unit(pair, &mut Environment::new());
                assert_eq!(
                    SRC.split_once("\n").unwrap().1.trim().replace("\r\n", "\n"),
                    format!("{}", module).trim(),
                );
            }
        };
    }

    test_file!(test_1, "tests/1.wgsl");
    test_file!(test_2, "tests/2.wgsl");
    test_file!(test_3, "tests/3.wgsl");
    test_file!(test_4, "tests/4.wgsl");
    test_file!(test_5, "tests/5.wgsl");
}
