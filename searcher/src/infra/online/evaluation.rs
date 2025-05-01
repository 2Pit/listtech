use crate::infra::online::program::*;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::{HashMap, VecDeque};

pub fn execute(program: &Program, ctx: &HashMap<String, f64>) -> Result<f64> {
    let mut stack: VecDeque<f64> = VecDeque::new();

    for op in &program.ops {
        match op {
            OpCode::PushNumber(n) => stack.push_back(*n),
            OpCode::PushVariable(name) => stack.push_back(*ctx.get(name).unwrap()), // todo fix
            OpCode::CallFunction { name, n_arg } => {
                let mut args = Vec::with_capacity(*n_arg);
                for _ in 0..*n_arg {
                    args.push(stack.pop_back().ok_or_else(|| anyhow!("Stack underflow"))?);
                }
                args.reverse();

                let result = match (name.as_str(), args.as_slice()) {
                    ("exp", [x]) => x.exp(),
                    ("log", [x]) => x.ln(),
                    ("sqrt", [x]) => x.sqrt(),
                    ("pow", [x, y]) => x.powf(*y),
                    _ => {
                        return Err(anyhow!(
                            "Unknown function or wrong args: {}({})",
                            name,
                            n_arg
                        ));
                    }
                };
                stack.push_back(result);
            }
        }
    }

    stack
        .pop_back()
        .ok_or_else(|| anyhow!("Stack empty after execution"))
}

#[test]
fn test_exp() {
    use crate::infra::online::parsing::Expr;

    let src = "exp(x)";
    let expr = Expr::parse(src).unwrap();
    let program = Program::compile_expr(expr);

    let ctx = HashMap::from_iter(vec![("x".to_string(), 3.0)]);

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
fn test_exp_pow() {
    use crate::infra::online::parsing::Expr;

    let src = "exp(pow(x,2))";
    let expr = Expr::parse(src).unwrap();
    let program = Program::compile_expr(expr);

    let ctx = HashMap::from_iter(vec![("x".to_string(), 3.0)]);

    let result = execute(&program, &ctx).unwrap();

    assert!(
        (result - (9f64).exp()).abs() < 1e-6,
        "Unexpected result: {result}"
    );
}
