use chumsky::prelude::*;
use std::collections::VecDeque;

// ===== AST =====

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expr> },
}

// ===== Парсер chumsky =====

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Simple<'a, char>>> {
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
                Expr::Number(s.parse().unwrap())
            });

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

// ===== Байт-код план =====

#[derive(Debug)]
pub enum OpCode {
    PushNumber(f64),
    PushVariable(String),
    CallFunction { name: String, argc: usize },
}

#[derive(Debug)]
pub struct Program {
    pub ops: Vec<OpCode>,
}

pub fn compile_expr(expr: &Expr) -> Program {
    let mut ops = Vec::new();
    compile_expr_rec(expr, &mut ops);
    Program { ops }
}

fn compile_expr_rec(expr: &Expr, ops: &mut Vec<OpCode>) {
    match expr {
        Expr::Number(n) => ops.push(OpCode::PushNumber(*n)),
        Expr::Variable(name) => ops.push(OpCode::PushVariable(name.clone())),
        Expr::FunctionCall { name, args } => {
            for arg in args {
                compile_expr_rec(arg, ops);
            }
            ops.push(OpCode::CallFunction {
                name: name.clone(),
                argc: args.len(),
            });
        }
    }
}

// ===== Исполнение =====

pub fn execute(program: &Program, ctx: &dyn Fn(&str) -> f64) -> Result<f64, String> {
    let mut stack: VecDeque<f64> = VecDeque::new();

    for op in &program.ops {
        match op {
            OpCode::PushNumber(n) => stack.push_back(*n),
            OpCode::PushVariable(name) => stack.push_back(ctx(name)),
            OpCode::CallFunction { name, argc } => {
                let mut args = Vec::with_capacity(*argc);
                for _ in 0..*argc {
                    args.push(stack.pop_back().ok_or("Stack underflow")?);
                }
                args.reverse(); // порядок аргументов важен!

                let result = match (name.as_str(), args.as_slice()) {
                    ("exp", [x]) => x.exp(),
                    ("log", [x]) => x.ln(),
                    ("sqrt", [x]) => x.sqrt(),
                    ("pow", [x, y]) => x.powf(*y),
                    // ("decay", [x, factor]) => x * (-factor).exp(),
                    _ => return Err(format!("Unknown function or wrong args: {name}({argc})")),
                };
                stack.push_back(result);
            }
        }
    }

    stack
        .pop_back()
        .ok_or_else(|| "Stack empty after execution".to_string())
}

#[test]
fn test_decay_1() {
    let src = "exp(x)";

    let expr = parser().parse(src).unwrap();

    let program = compile_expr(&expr);

    let ctx = |var: &str| match var {
        "x" => -3.0,
        _ => 0.0,
    };

    let result = execute(&program, &ctx).unwrap();

    fn exp(x: f64) -> f64 {
        x.exp()
    }

    assert!(
        (result - exp(-3.0)).abs() < 1e-6,
        "Unexpected result: {result}"
    );
}

#[test]
fn test_decay_expression_execution() {
    let src = "exp(pow(x,2))";

    let expr = parser().parse(src).unwrap();

    let program = compile_expr(&expr);

    let ctx = |var: &str| match var {
        "x" => 3.0,
        _ => 0.0,
    };

    let result = execute(&program, &ctx).unwrap();

    assert!(
        (result - (9f64).exp()).abs() < 1e-6,
        "Unexpected result: {result}"
    );
}
