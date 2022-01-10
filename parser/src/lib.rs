use std::collections::HashMap;
use std::sync::Arc;

use ast::types::{DataType, ScalarType};
use ast::{
    AccessMode, AssignmentLhs, AssignmentLhsPostfix, AttrList, BinOp, Expr, ExprNode, FnAttr,
    FnDecl, FnInput, FnOutput, GlobalVarAttr, GlobalVarDecl, Lit, Module, ShaderStage, Statement,
    StorageClass, StructDecl, StructMember, UnOp, VarQualifier,
};
use peeking_take_while::PeekableExt;
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct WGSLParser;

type Environment = HashMap<String, DataType>;

pub fn parse(input: &str) -> Module {
    let pairs = WGSLParser::parse(Rule::translation_unit, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_translation_unit(pair)
}

fn parse_translation_unit(pair: Pair<Rule>) -> Module {
    let decls = pair
        .into_inner()
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(parse_global_decl)
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

fn parse_global_decl(pair: Pair<Rule>) -> GlobalDecl {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::global_variable_decl => GlobalDecl::Var(parse_global_variable_decl(pair)),
        Rule::struct_decl => GlobalDecl::Struct(parse_struct_decl(pair)),
        Rule::function_decl => GlobalDecl::Fn(parse_function_decl(pair)),
        _ => unreachable!(),
    }
}

fn parse_global_variable_decl(pair: Pair<Rule>) -> GlobalVarDecl {
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

    GlobalVarDecl {
        attrs,
        qualifier,
        name,
        data_type: data_type.unwrap_or_else(|| {
            expr.as_ref()
                .expect("var declaration must have type or initializer")
                .data_type
                .clone()
        }),
        initializer: expr,
    }
}

fn parse_struct_decl(pair: Pair<Rule>) -> StructDecl {
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

    StructDecl { name, members }
}

fn parse_function_decl(pair: Pair<Rule>) -> FnDecl {
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

    let mut env = Environment::new();
    for param in &inputs {
        env.insert(param.name.clone(), param.data_type.clone());
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
        _ => unreachable!(),
    }
}

fn parse_let_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    let expression = parse_expression(pairs.next().unwrap(), env);
    env.insert(ident.clone(), expression.data_type.clone());
    Statement::LetDecl(ident, expression)
}

fn parse_var_statement(pair: Pair<Rule>, env: &mut Environment) -> Statement {
    let mut pairs = pair.into_inner();
    let ident = pairs.next().unwrap().as_str().to_owned();
    // TODO: Rhs for var is optional in grammar but not in the AST
    let expression = parse_expression(pairs.next().unwrap(), env);
    env.insert(ident.clone(), expression.data_type.clone());
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
                Rule::expression => AssignmentLhsPostfix::ArrayIndex(parse_expression(pair, env)),
                Rule::ident => AssignmentLhsPostfix::Member(pair.as_str().to_owned()),
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
        Rule::primary_expression => parse_primary_expression(pair, env),
        Rule::infix_expression => parse_infix_expression(pair, env),
        _ => unreachable!(),
    }
}

fn parse_infix_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pairs = pair.into_inner();

    let primary = |pair| parse_primary_expression(pair, env);
    let infix = |l: ExprNode, op: Pair<Rule>, r: ExprNode| -> ExprNode {
        let op: BinOp = op.as_rule().into();

        ExprNode {
            data_type: op.type_eval(&l.data_type, &r.data_type),
            expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
        }
    };

    precedence_table().climb(pairs, primary, infix)
}

fn parse_primary_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::literal_expression => parse_literal_expression(pair),
        Rule::type_cons_expression => parse_type_cons_expression(pair, env),
        Rule::call_expression => todo!(),
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
            DataType::Array(Arc::new(parse_type_decl(pair)))
        }
        Rule::ident => DataType::User(Arc::new(pair.as_str().to_owned())),
        _ => panic!("{}", pair),
    }
}

fn parse_var_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    ExprNode {
        data_type: env[pair.as_str()].clone(),
        expr: Expr::Var(pair.as_str().to_owned()),
    }
}

fn parse_paren_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let pair = pair.into_inner().next().unwrap();
    parse_expression(pair, env)
}

fn parse_unary_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    let mut pairs = pair.into_inner();
    let op = pairs.next().unwrap();
    let expr = pairs.next().unwrap();

    let op = match op.as_rule() {
        Rule::op_minus => UnOp::Neg,
        Rule::op_log_not => UnOp::Not,
        Rule::op_bit_not => UnOp::BitNot,
        _ => unreachable!(),
    };

    let expr = parse_primary_expression(expr, env);

    ExprNode {
        data_type: op.type_eval(&expr.data_type),
        expr: Expr::UnOp(op, Box::new(expr)),
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
                let module = parse_translation_unit(pair);
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
