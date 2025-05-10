use chumsky::prelude::*;

use chumsky::error::Simple;

#[derive(Debug)]
pub enum Expr {
    Number(f32),
    Variable(String),
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Expr {
    pub fn parse(input: &str) -> ParseResult<Self, Simple<char>> {
        build_parser().parse(input)
    }
}

fn build_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Simple<'a, char>>> {
    recursive(|expr| {
        let number = text::int(10)
            .then(
                just('.')
                    .ignore_then(text::digits(10).collect::<String>())
                    .or_not(),
            )
            .map(|(int_part, frac_part): (&str, Option<String>)| {
                let mut s = int_part.to_string();
                if let Some(frac) = frac_part {
                    s.push('.');
                    s.push_str(&frac);
                }
                Expr::Number(s.parse::<f32>().unwrap())
            })
            .padded();

        let ident = text::ident()
            .map(|s: &str| Expr::Variable(s.to_string()))
            .padded();

        let args = expr
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<Expr>>()
            .delimited_by(just('(').padded(), just(')').padded())
            .padded();

        let func_call = text::ident()
            .then(args)
            .map(|(name, args): (&str, Vec<Expr>)| Expr::FunctionCall {
                name: name.to_string(),
                args,
            })
            .padded();

        let atom = choice((func_call, ident, number))
            .or(expr
                .clone()
                .delimited_by(just('(').padded(), just(')').padded()))
            .padded();

        let unary = just('-')
            .or_not()
            .then(atom)
            .map(
                |(maybe_minus, expr): (Option<char>, Expr)| match maybe_minus {
                    Some(_) => Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        expr: Box::new(expr),
                    },
                    None => expr,
                },
            )
            .padded();

        let op_mul_div = just('*')
            .padded()
            .to(BinaryOp::Mul)
            .or(just('/').padded().to(BinaryOp::Div))
            .map(|op: BinaryOp| op)
            .padded();

        let product = unary
            .clone()
            .foldl(op_mul_div.then(unary).repeated(), |lhs, (op, rhs)| {
                Expr::BinaryOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            })
            .padded();

        let op_add_sub = just('+')
            .padded()
            .to(BinaryOp::Add)
            .or(just('-').padded().to(BinaryOp::Sub))
            .map(|op: BinaryOp| op)
            .padded();

        let sum = product
            .clone()
            .foldl(op_add_sub.then(product).repeated(), |lhs, (op, rhs)| {
                Expr::BinaryOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            });

        sum
    })
    .then_ignore(end())
}
