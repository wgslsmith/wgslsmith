use std::collections::HashMap;

use ast::types::{DataType, ScalarType};
use ast::{BinOp, Expr, ExprNode, Lit, UnOp};
use pest::iterators::Pair;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct WGSLParser;

type Environment<'a> = HashMap<&'a str, DataType>;

pub fn parse(input: &str) -> ExprNode {
    let env = Environment::new();
    let pairs = WGSLParser::parse(Rule::expression, input).unwrap();
    let pair = pairs.into_iter().next().unwrap();
    parse_expression(pair, &env)
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

    fn parse_t_scalar(pair: Pair<Rule>) -> ScalarType {
        pair.into_inner().next().unwrap().as_rule().into()
    }

    let t = match t_decl.as_rule() {
        Rule::t_scalar => DataType::Scalar(parse_t_scalar(t_decl)),
        Rule::t_vector => {
            let t_vector = t_decl.into_inner().next().unwrap();
            let n = match t_vector.as_rule() {
                Rule::t_vec2 => 2,
                Rule::t_vec3 => 3,
                Rule::t_vec4 => 4,
                _ => unreachable!(),
            };

            DataType::Vector(n, parse_t_scalar(t_vector.into_inner().next().unwrap()))
        }
        _ => panic!("{}", t_decl),
    };

    let args = pairs.map(|pair| parse_expression(pair, env)).collect();

    ExprNode {
        data_type: t,
        expr: Expr::TypeCons(t, args),
    }
}

fn parse_var_expression(pair: Pair<Rule>, env: &Environment) -> ExprNode {
    ExprNode {
        data_type: env[pair.as_str()],
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

    /// Helper macro to compare the input with the parsed AST by converting the AST back to its
    /// canonical code representation.
    macro_rules! assert_parses {
        ($input:expr) => {
            assert_parses!($input, $input)
        };
        ($input:expr, $expected:expr) => {
            assert_eq!(format!("{}", parse($input)), $expected)
        };
    }

    #[test]
    fn bool_literals() {
        assert_parses!("true");
        assert_parses!("false");
    }

    #[test]
    fn int_literals() {
        assert_parses!("123");
        assert_parses!("-123");
        assert_parses!("123u");
    }

    #[test]
    fn type_cons_expression() {
        assert_parses!("bool(true)");
        assert_parses!("i32(1)");
        assert_parses!("u32(1u)");
        assert_parses!("vec2<i32>(1, 2)");
        assert_parses!("vec3<i32>(1, 2, 3)");
        assert_parses!("vec4<i32>(1, 2, 3, 4)");
    }

    #[test]
    fn paren_expression() {
        assert_parses!("(123)", "123");
        assert_parses!("(true)", "true");
    }

    #[test]
    fn unary_expression() {
        assert_parses!("!true", "!(true)");
        assert_parses!("!(false)", "!(false)");
        assert_parses!("~123", "~(123)");
        assert_parses!("- 123", "-(123)");
    }

    #[test]
    fn binary_expression() {
        assert_parses!("1 + 2", "(1) + (2)");
        assert_parses!("1 + 2 * 3", "(1) + ((2) * (3))");
        assert_parses!("(1 + 2) * 3", "((1) + (2)) * (3)");
    }
}
