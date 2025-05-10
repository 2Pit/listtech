pub mod evaluation;
pub mod parsing;
pub mod program;

#[cfg(test)]
mod tests {
    use crate::infra::online::{evaluation::execute, parsing::Expr, program::Program};

    fn exec(src: &str, ctx_map: &[(&str, f32)]) -> f32 {
        let expr = Expr::parse(src).unwrap();
        let program = Program::compile_expr(expr);

        let mut ctx_names = Vec::new();
        let mut ctx_values = Vec::new();
        for (k, v) in ctx_map {
            ctx_names.push(k.to_string());
            ctx_values.push(*v);
        }

        execute(&program, &ctx_values).unwrap()
    }

    #[test]
    fn test_exp() {
        let result = exec("exp(x)", &[("x", 3.0)]);
        assert!(
            (result - 3.0f32.exp()).abs() < 1e-6,
            "Unexpected result: {result}"
        );
    }

    #[test]
    fn test_exp_pow() {
        let result = exec("exp(pow(x,2))", &[("x", 3.0)]);
        assert!(
            (result - (9.0f32).exp()).abs() < 1e-6,
            "Unexpected result: {result}"
        );
    }

    #[test]
    fn test_expr_execution_variants() {
        let cases = vec![
            ("1 + 2", 3.0, &[][..]),
            ("4 - 2", 2.0, &[][..]),
            ("3 * 5", 15.0, &[][..]),
            ("10 / 2", 5.0, &[][..]),
            ("1 + 2 * 3", 7.0, &[][..]),
            ("(1 + 2) * 3", 9.0, &[][..]),
            ("-5", -5.0, &[][..]),
            ("-(2 + 3)", -5.0, &[][..]),
            ("-x", -4.0, &[("x", 4.0)]),
            ("exp(1 + 1)", (2.0f32).exp(), &[][..]),
            ("exp(ln(10))", 10.0, &[][..]),
            ("sqrt(16 + 9)", 5.0, &[][..]),
            ("pow(2, 3) + 1", 9.0, &[][..]),
            ("x + y", 9.0, &[("x", 4.0), ("y", 5.0)]),
            ("pow(x, 2) + pow(y, 2)", 25.0, &[("x", 3.0), ("y", 4.0)]),
            (
                "sqrt(pow(x, 2) + pow(y, 2))",
                5.0,
                &[("x", 3.0), ("y", 4.0)],
            ),
            (
                "-sqrt(pow(x,2) + pow(y,2))",
                -5.0,
                &[("x", 3.0), ("y", 4.0)],
            ),
        ];

        for (src, expected, ctx_map) in cases {
            let result = exec(src, ctx_map);
            assert!(
                (result - expected).abs() < 1e-6,
                "Expr `{src}` → expected {expected}, got {result}"
            );
        }
    }

    #[test]
    fn test_multiple_variable_usage() {
        let cases: Vec<(&str, f32, &[(&str, f32)])> = vec![
            ("x + x", 6.0, &[("x", 3.0)]),
            ("x + y + x", 11.0, &[("x", 4.0), ("y", 3.0)]),
            ("x * x + y", 19.0, &[("x", 4.0), ("y", 3.0)]),
            ("pow(x, 2) + pow(x, 2)", 50.0, &[("x", 5.0)]),
            ("x + y + z + x", 18.0, &[("x", 5.0), ("y", 4.0), ("z", 4.0)]),
            ("x * y + x * y", 30.0, &[("x", 3.0), ("y", 5.0)]),
            ("pow(x + y, 2)", 49.0, &[("x", 3.0), ("y", 4.0)]),
            ("sqrt(x * x + x * x)", (18.0 as f32).sqrt(), &[("x", 3.0)]),
            ("exp(x) + exp(x)", 2.0 * 3.0f32.exp(), &[("x", 3.0)]),
        ];

        for (src, expected, ctx_map) in cases {
            let result = exec(src, ctx_map);
            assert!(
                (result - expected).abs() < 1e-5,
                "Expr `{src}` → expected {expected}, got {result}"
            );
        }
    }
}
