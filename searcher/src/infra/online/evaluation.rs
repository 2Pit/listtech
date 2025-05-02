use crate::infra::online::program::*;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::{HashMap, VecDeque};

pub fn execute(program: &Program, ctx: &HashMap<String, f64>) -> Result<f64> {
    let mut stack: VecDeque<f64> = VecDeque::new();

    for op in &program.ops {
        match op {
            OpCode::PushNumber(n) => stack.push_back(*n),

            OpCode::PushVariable(name) => {
                let value = ctx
                    .get(name)
                    .ok_or_else(|| anyhow!("Unknown variable: {name}"))?;
                stack.push_back(*value);
            }

            OpCode::CallFunction { name, n_arg } => {
                let mut args = Vec::with_capacity(*n_arg);
                for _ in 0..*n_arg {
                    args.push(stack.pop_back().ok_or_else(|| anyhow!("Stack underflow"))?);
                }
                args.reverse();

                let result = match (name.as_str(), args.as_slice()) {
                    // унарные
                    ("exp", [x]) => Ok(x.exp()),
                    ("ln", [x]) => Ok(x.ln()),
                    ("sqrt", [x]) => Ok(x.sqrt()),

                    // бинарные
                    ("pow", [x, y]) => Ok(x.powf(*y)),
                    ("+", [x, y]) => Ok(x + y),
                    ("-", [x, y]) => Ok(x - y),
                    ("*", [x, y]) => Ok(x * y),
                    ("/", [x, y]) => Ok(x / y),

                    _ => Err(anyhow!(
                        "Unknown function or wrong args: {}({})",
                        name,
                        n_arg
                    )),
                }?;

                stack.push_back(result);
            }
        }
    }

    stack
        .pop_back()
        .ok_or_else(|| anyhow!("Stack empty after execution"))
}
