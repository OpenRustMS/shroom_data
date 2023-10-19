use std::str::FromStr;

use pest::{iterators::Pairs, pratt_parser::PrattParser, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "eval.pest"]
pub struct EvalParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
    };
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Subtract => write!(f, "-"),
            Op::Multiply => write!(f, "*"),
            Op::Divide => write!(f, "/"),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    Integer(i32),
    Var(char),
    Ceil(Box<Expr>),
    Floor(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

impl FromStr for Expr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse = EvalParser::parse(Rule::expr, s)?;
        Ok(parse_expr(parse))
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(i) => write!(f, "{}", i),
            Expr::Var(c) => write!(f, "{}", c),
            Expr::Ceil(expr) => write!(f, "u({})", expr),
            Expr::Floor(expr) => write!(f, "d({})", expr),
            Expr::BinOp { lhs, op, rhs } => write!(f, "({}{}{})", lhs, op, rhs),
        }
    }
}

impl Expr {}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::var => Expr::Var('x'),
            Rule::floor => Expr::Floor(Box::new(parse_expr(primary.into_inner()))),
            Rule::ceil => Expr::Ceil(Box::new(parse_expr(primary.into_inner()))),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .parse(pairs)
}

pub struct EvalContext {
    pub x: i32,
}

impl EvalContext {
    pub fn new(x: i32) -> Self {
        Self { x }
    }

    pub fn eval(&self, expr: &Expr) -> f32 {
        match expr {
            Expr::Integer(i) => *i as f32,
            Expr::Var(_c) => self.x as f32,
            Expr::Ceil(expr) => self.eval(expr.as_ref()).ceil(),
            Expr::Floor(expr) => self.eval(expr.as_ref()).floor(),
            Expr::BinOp { lhs, op, rhs } => {
                let lhs = self.eval(lhs);
                let rhs = self.eval(rhs);

                match op {
                    Op::Add => lhs + rhs,
                    Op::Subtract => lhs - rhs,
                    Op::Multiply => lhs * rhs,
                    Op::Divide => lhs / rhs,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn parse() {
        let parse = EvalParser::parse(Rule::expr, "6+2*u(x/5)+d(1)-5").unwrap();
        let expr = parse_expr(parse);
        assert_eq!(EvalContext::new(0).eval(&expr), 2.);
        assert_eq!(expr.to_string(), "(((6+(2*u((x/5))))+d(1))-5)");
    }
}
