use chumsky::prelude::*;

use chumsky::error::Simple;

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expr> },
}

impl Expr {
    pub fn parse(input: &str) -> ParseResult<Self, Simple<char>> {
        build_parser().parse(input)
    }
}

fn build_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Simple<'a, char>>> {
    recursive(|expr| {
        let number = just('-')
            .or_not()
            .then(text::int(10))
            .then(
                just('.')
                    .ignore_then(text::digits(10).collect::<String>())
                    .or_not(),
            )
            .map(
                |((sign, int_part), frac_part): ((Option<char>, &str), Option<String>)| {
                    let s_num = format!(
                        "{}{}.{}",
                        sign.map(|s| s.to_string()).unwrap_or("".to_string()),
                        int_part,
                        frac_part.unwrap_or("".to_string())
                    );
                    Expr::Number(s_num.parse().unwrap())
                },
            );

        let ident = text::ident()
            .map(|s: &str| s.to_string())
            .map(Expr::Variable);

        let args = expr
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('('), just(')'));

        let func_call = text::ident()
            .then(args)
            .map(|(name, args): (&str, Vec<Expr>)| Expr::FunctionCall {
                name: name.to_string(),
                args,
            });

        choice((func_call.or(ident), number))
    })
    .then_ignore(end())
}
