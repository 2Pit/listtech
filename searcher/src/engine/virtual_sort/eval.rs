use crate::engine::virtual_sort::program::{OpCode, Program};
use anyhow::{Result, anyhow};
use std::collections::VecDeque;

pub fn execute(program: &Program, ctx: &[f32]) -> Result<f32> {
    let mut stack: VecDeque<f32> = VecDeque::new();

    for op in &program.ops {
        match op {
            OpCode::PushNumber(n) => stack.push_back(*n),

            OpCode::PushVariable(idx) => {
                let value = ctx
                    .get(*idx)
                    .ok_or_else(|| anyhow!("Variable index out of bounds: {}", idx))?;
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
