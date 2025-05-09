use crate::infra::online::parsing::Expr;

use super::parsing::{BinaryOp, UnaryOp};

#[derive(Debug, Clone)]
pub struct Program {
    pub ops: Vec<OpCode>,
}

#[derive(Debug, Clone)]
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

            Expr::UnaryOp { op, expr } => {
                Program::compile_expr_rec(*expr, ops);
                match op {
                    UnaryOp::Neg => {
                        ops.push(OpCode::PushNumber(-1.0));
                        ops.push(OpCode::CallFunction {
                            name: "*".to_string(),
                            n_arg: 2,
                        });
                    }
                }
            }

            Expr::BinaryOp { op, lhs, rhs } => {
                Program::compile_expr_rec(*lhs, ops);
                Program::compile_expr_rec(*rhs, ops);
                let name = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                };
                ops.push(OpCode::CallFunction {
                    name: name.to_string(),
                    n_arg: 2,
                });
            }
        }
    }
}
