use crate::infra::online::parsing::Expr;

#[derive(Debug)]
pub struct Program {
    pub ops: Vec<OpCode>,
}

#[derive(Debug)]
pub enum OpCode {
    PushNumber(f64),
    PushVariable(String),
    CallFunction { name: String, n_arg: usize },
}

impl Program {
    pub fn compile_expr(expr: Expr) -> Program {
        let mut ops = Vec::new();
        Program::compile_expr_rec(expr, &mut ops);
        Program { ops }
    }

    fn compile_expr_rec(expr: Expr, ops: &mut Vec<OpCode>) {
        match expr {
            Expr::Number(n) => ops.push(OpCode::PushNumber(n)),
            Expr::Variable(name) => ops.push(OpCode::PushVariable(name)),
            Expr::FunctionCall { name, args } => {
                let args_len = args.len();
                for arg in args {
                    Program::compile_expr_rec(arg, ops);
                }
                ops.push(OpCode::CallFunction {
                    name,
                    n_arg: args_len,
                });
            }
        }
    }
}
