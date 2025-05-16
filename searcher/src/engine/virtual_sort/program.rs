use crate::engine::virtual_sort::expr::{BinaryOp, Expr, UnaryOp};

#[derive(Debug, Clone)]
pub enum OpCode {
    PushNumber(f32),
    PushVariable(usize), // индекс в env
    CallFunction { name: String, n_arg: usize },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub ops: Vec<OpCode>,
    pub env: Vec<String>, // список переменных
}

impl Program {
    pub fn compile_expr(expr: Expr) -> Self {
        let mut ops = Vec::new();
        let mut env = Vec::new();
        Self::compile_expr_rec(expr, &mut ops, &mut env);
        Self { ops, env }
    }

    fn compile_expr_rec(expr: Expr, ops: &mut Vec<OpCode>, env: &mut Vec<String>) {
        match expr {
            Expr::Number(n) => ops.push(OpCode::PushNumber(n)),

            Expr::Variable(name) => {
                tracing::debug!(var = %name, "Compiling Expr::Variable");
                let idx = match env.iter().position(|v| v == &name) {
                    Some(i) => i,
                    None => {
                        env.push(name.clone());
                        env.len() - 1
                    }
                };
                ops.push(OpCode::PushVariable(idx));
            }

            // Специальная обработка now_ms()
            Expr::FunctionCall { ref name, ref args } if name == "now_ms" && args.is_empty() => {
                let now_ms = chrono::Utc::now().timestamp_millis() as f32;
                tracing::debug!(now_ms, "Replacing now_ms() with constant");
                ops.push(OpCode::PushNumber(now_ms));
            }

            Expr::FunctionCall { name, args } => {
                let args_len = args.len();
                for arg in args {
                    Self::compile_expr_rec(arg, ops, env);
                }
                ops.push(OpCode::CallFunction {
                    name,
                    n_arg: args_len,
                });
            }

            Expr::UnaryOp { op, expr } => {
                Self::compile_expr_rec(*expr, ops, env);
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
                Self::compile_expr_rec(*lhs, ops, env);
                Self::compile_expr_rec(*rhs, ops, env);
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
