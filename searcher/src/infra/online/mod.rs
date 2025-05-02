pub mod evaluation;
pub mod parsing;
pub mod program;

#[cfg(test)]
mod tests {
    use crate::infra::online::{evaluation::execute, program::Program};
    use std::collections::HashMap;

    #[allow(unused_macros)]
    macro_rules! hashmap {
        ($($k:expr => $v:expr),* $(,)?) => {{
            let mut m = HashMap::new();
            $( m.insert($k.to_string(), $v); )*
            m
        }};
    }

    #[test]
    fn test_exp() {
        use crate::infra::online::parsing::Expr;

        let src = "exp(x)";
        let expr = Expr::parse(src).unwrap();
        let program = Program::compile_expr(expr);
        let ctx = hashmap! { "x" => 3.0};

        let result = execute(&program, &ctx).unwrap();

        fn exp(x: f64) -> f64 {
            x.exp()
        }

        assert!(
            (result - exp(3.0)).abs() < 1e-6,
            "Unexpected result: {result}"
        );
    }

    #[test]
    fn test_exp_pow() {
        use crate::infra::online::parsing::Expr;

        let src = "exp(pow(x,2))";
        let expr = Expr::parse(src).unwrap();
        let program = Program::compile_expr(expr);

        let ctx = hashmap! { "x" => 3.0};

        let result = execute(&program, &ctx).unwrap();

        assert!(
            (result - (9f64).exp()).abs() < 1e-6,
            "Unexpected result: {result}"
        );
    }

    #[test]
    fn test_expr_execution_variants() {
        use crate::infra::online::parsing::Expr;
        use crate::infra::online::program::Program;
        let empty_map = HashMap::new();

        let cases = vec![
            ("1 + 2", 3.0, empty_map.clone()),
            ("4 - 2", 2.0, empty_map.clone()),
            ("3 * 5", 15.0, empty_map.clone()),
            ("10 / 2", 5.0, empty_map.clone()),
            ("1 + 2 * 3", 7.0, empty_map.clone()),
            ("(1 + 2) * 3", 9.0, empty_map.clone()),
            ("-5", -5.0, empty_map.clone()),
            ("-(2 + 3)", -5.0, empty_map.clone()),
            ("-x", -4.0, hashmap! { "x" => 4.0 }),
            ("exp(1 + 1)", (2.0f64).exp(), empty_map.clone()),
            ("exp(ln(10))", 10.0, empty_map.clone()),
            ("sqrt(16 + 9)", 5.0, empty_map.clone()),
            ("pow(2, 3) + 1", 9.0, empty_map.clone()),
            ("x + y", 9.0, hashmap! { "x" => 4.0, "y" => 5.0 }),
            (
                "pow(x, 2) + pow(y, 2)",
                25.0,
                hashmap! { "x" => 3.0, "y" => 4.0 },
            ),
            (
                "sqrt(pow(x, 2) + pow(y, 2))",
                5.0,
                hashmap! { "x" => 3.0, "y" => 4.0 },
            ),
            (
                "-sqrt(pow(x,2) + pow(y,2))",
                -5.0,
                hashmap! { "x" => 3.0, "y" => 4.0 },
            ),
        ];

        for (src, expected, ctx) in cases {
            let expr = Expr::parse(src).unwrap();
            let program = Program::compile_expr(expr);

            let result = execute(&program, &ctx).unwrap();
            assert!(
                (result - expected).abs() < 1e-6,
                "Expr `{src}` → expected {expected}, got {result}"
            );
        }
    }
}
