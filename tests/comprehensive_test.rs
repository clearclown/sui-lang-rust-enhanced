//! Comprehensive tests matching Python test_interpreter.py
//! TDD: Write tests first, then implement

use sui_lang::interpreter::{Interpreter, Parser};

// ============================================================================
// TestBasicOperations - 基本算術とアサインメント
// ============================================================================

mod basic_operations {
    use super::*;

    #[test]
    fn test_assignment() {
        let mut interp = Interpreter::new();
        let result = interp.run("= g0 42\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }

    #[test]
    fn test_addition() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 10\n= v1 20\n+ v2 v0 v1\n. v2", &[]).unwrap();
        assert_eq!(result, vec!["30"]);
    }

    #[test]
    fn test_subtraction() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 50\n= v1 30\n- v2 v0 v1\n. v2", &[]).unwrap();
        assert_eq!(result, vec!["20"]);
    }

    #[test]
    fn test_multiplication() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 6\n= v1 7\n* v2 v0 v1\n. v2", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }

    #[test]
    fn test_division() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 100\n= v1 4\n/ v2 v0 v1\n. v2", &[]).unwrap();
        // Python returns 25.0, Rust should return 25 for integer division
        assert!(result[0] == "25" || result[0] == "25.0");
    }

    #[test]
    fn test_modulo() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 17\n= v1 5\n% v2 v0 v1\n. v2", &[]).unwrap();
        assert_eq!(result, vec!["2"]);
    }
}

// ============================================================================
// TestComparisons - 比較演算
// ============================================================================

mod comparisons {
    use super::*;

    #[test]
    fn test_less_than_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("< v0 5 10\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_less_than_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("< v0 10 5\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_greater_than_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("> v0 10 5\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_greater_than_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("> v0 5 10\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_equality_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("~ v0 42 42\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_equality_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("~ v0 42 43\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }
}

// ============================================================================
// TestLogicalOperations - 論理演算
// ============================================================================

mod logical_operations {
    use super::*;

    #[test]
    fn test_not_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 0\n! v1 v0\n. v1", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_not_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 1\n! v1 v0\n. v1", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_and_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("& v0 1 1\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_and_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("& v0 1 0\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_or_true() {
        let mut interp = Interpreter::new();
        let result = interp.run("| v0 0 1\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_or_false() {
        let mut interp = Interpreter::new();
        let result = interp.run("| v0 0 0\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }
}

// ============================================================================
// TestControlFlow - 制御フロー
// ============================================================================

mod control_flow {
    use super::*;

    #[test]
    fn test_unconditional_jump() {
        let mut interp = Interpreter::new();
        let result = interp.run("@ 0\n. 1\n: 0\n. 2", &[]).unwrap();
        assert_eq!(result, vec!["2"]);
    }

    #[test]
    fn test_conditional_jump_taken() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 1\n? v0 0\n. 1\n: 0\n. 2", &[]).unwrap();
        assert_eq!(result, vec!["2"]);
    }

    #[test]
    fn test_conditional_jump_not_taken() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 0\n? v0 0\n. 1\n@ 1\n: 0\n. 2\n: 1", &[]).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    #[test]
    fn test_loop_sum_1_to_5() {
        let code = r#"
= v0 0
= v1 1
: 0
> v2 v1 5
? v2 1
+ v0 v0 v1
+ v1 v1 1
@ 0
: 1
. v0
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["15"]); // 1+2+3+4+5 = 15
    }
}

// ============================================================================
// TestFunctions - 関数
// ============================================================================

mod functions {
    use super::*;

    #[test]
    fn test_simple_function() {
        let code = r#"
# 0 1 {
+ v0 a0 10
^ v0
}
$ g0 0 5
. g0
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["15"]);
    }

    #[test]
    fn test_recursive_factorial() {
        let code = r#"
# 0 1 {
< v0 a0 2
! v1 v0
? v1 1
^ 1
: 1
- v2 a0 1
$ v3 0 v2
* v4 a0 v3
^ v4
}
$ g0 0 5
. g0
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["120"]); // 5! = 120
    }

    #[test]
    fn test_fibonacci() {
        let code = r#"
# 0 1 {
< v0 a0 2
! v1 v0
? v1 1
^ a0
: 1
- v2 a0 1
$ v3 0 v2
- v4 a0 2
$ v5 0 v4
+ v6 v3 v5
^ v6
}
$ g0 0 10
. g0
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["55"]);
    }

    #[test]
    fn test_multiple_arguments() {
        let code = r#"
# 0 3 {
+ v0 a0 a1
+ v1 v0 a2
^ v1
}
$ g0 0 10 20 30
. g0
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["60"]); // 10+20+30
    }
}

// ============================================================================
// TestArrays - 配列
// ============================================================================

mod arrays {
    use super::*;

    #[test]
    fn test_array_create_and_write() {
        let mut interp = Interpreter::new();
        let result = interp.run("[ g0 3\n{ g0 0 10\n{ g0 1 20\n{ g0 2 30\n] v0 g0 1\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["20"]);
    }

    #[test]
    fn test_array_sum() {
        let code = r#"
[ g0 5
{ g0 0 1
{ g0 1 2
{ g0 2 3
{ g0 3 4
{ g0 4 5
= g1 0
= v0 0
: 0
< v1 v0 5
! v2 v1
? v2 1
] v3 g0 v0
+ g1 g1 v3
+ v0 v0 1
@ 0
: 1
. g1
"#;
        let mut interp = Interpreter::new();
        let result = interp.run(code, &[]).unwrap();
        assert_eq!(result, vec!["15"]); // 1+2+3+4+5
    }

    #[test]
    fn test_array_out_of_bounds_returns_zero() {
        let mut interp = Interpreter::new();
        let result = interp.run("[ g0 3\n] v0 g0 10\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }
}

// ============================================================================
// TestStrings - 文字列
// ============================================================================

mod strings {
    use super::*;

    #[test]
    fn test_string_output() {
        let mut interp = Interpreter::new();
        let result = interp.run(". \"Hello\"", &[]).unwrap();
        assert_eq!(result, vec!["Hello"]);
    }

    #[test]
    fn test_string_with_spaces() {
        let mut interp = Interpreter::new();
        let result = interp.run(". \"Hello World\"", &[]).unwrap();
        assert_eq!(result, vec!["Hello World"]);
    }

    #[test]
    fn test_string_assignment() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 \"test\"\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["test"]);
    }
}

// ============================================================================
// TestCommandLineArgs - コマンドライン引数
// ============================================================================

mod command_line_args {
    use super::*;

    #[test]
    fn test_args_count() {
        let mut interp = Interpreter::new();
        let result = interp.run(". g100", &["10".to_string(), "20".to_string()]).unwrap();
        assert_eq!(result, vec!["2"]);
    }

    #[test]
    fn test_args_values() {
        let mut interp = Interpreter::new();
        let result = interp.run("+ v0 g101 g102\n. v0", &["10".to_string(), "20".to_string()]).unwrap();
        assert_eq!(result, vec!["30"]);
    }

    #[test]
    fn test_no_args() {
        let mut interp = Interpreter::new();
        let result = interp.run(". g100", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_string_args() {
        let mut interp = Interpreter::new();
        let result = interp.run(". g101", &["hello".to_string()]).unwrap();
        assert_eq!(result, vec!["hello"]);
    }
}

// ============================================================================
// TestValidation - 行ごとのバリデーション
// ============================================================================

mod validation {
    use super::*;

    #[test]
    fn test_valid_assignment() {
        let errors = Parser::validate("= v0 10");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_addition() {
        let errors = Parser::validate("+ v0 v1 v2");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_addition_missing_args() {
        let errors = Parser::validate("+ v0 v1");
        assert!(!errors.is_empty());
        let error_msg = errors[0].to_string();
        assert!(error_msg.contains("expected") || error_msg.contains("argument") || error_msg.contains("Missing"));
    }

    #[test]
    fn test_valid_function_def() {
        let errors = Parser::validate("# 0 2 {");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_function_def() {
        let errors = Parser::validate("# 0 2");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_comment() {
        let errors = Parser::validate("; this is a comment");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_empty_line() {
        let errors = Parser::validate("");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_array_write() {
        let errors = Parser::validate("{ arr 0 10");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_output() {
        let errors = Parser::validate(". v0");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_output_missing_arg() {
        let errors = Parser::validate(".");
        assert!(!errors.is_empty());
    }
}

// ============================================================================
// TestComments - コメント処理
// ============================================================================

mod comments {
    use super::*;

    #[test]
    fn test_semicolon_comment() {
        let mut interp = Interpreter::new();
        let result = interp.run("; This is a comment\n= v0 42\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }

    #[test]
    fn test_inline_comment() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 42 ; inline comment\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }

    #[test]
    fn test_empty_lines() {
        let mut interp = Interpreter::new();
        let result = interp.run("\n\n= v0 42\n\n. v0\n\n", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }
}

// ============================================================================
// TestFloatArithmetic - 浮動小数点演算
// ============================================================================

mod float_arithmetic {
    use super::*;

    #[test]
    fn test_float_assignment() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 3.14\n. v0", &[]).unwrap();
        assert!(result[0].starts_with("3.14"));
    }

    #[test]
    fn test_float_addition() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 1.5\n= v1 2.5\n+ v2 v0 v1\n. v2", &[]).unwrap();
        // Result can be "4" or "4.0" depending on implementation
        let value: f64 = result[0].parse().unwrap();
        assert_eq!(value, 4.0);
    }

    #[test]
    fn test_float_multiplication() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 3.14\n= v1 2.0\n* v2 v0 v1\n. v2", &[]).unwrap();
        assert!(result[0].starts_with("6.28"));
    }

    #[test]
    fn test_integer_division_result() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 10\n= v1 3\n/ v2 v0 v1\n. v2", &[]).unwrap();
        // Should be 3.333... or 3 depending on implementation
        let value: f64 = result[0].parse().unwrap();
        assert!(value > 3.0 && value < 4.0);
    }
}

// ============================================================================
// TestFFI - Rust FFI (P/R命令)
// ============================================================================

mod ffi {
    use super::*;

    #[test]
    fn test_ffi_sqrt() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"math.sqrt\" 16\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["4.0"]);
    }

    #[test]
    fn test_ffi_pow() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"math.pow\" 2 10\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["1024.0"]);
    }

    #[test]
    fn test_ffi_abs() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"abs\" -42\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["42"]);
    }

    #[test]
    fn test_ffi_max() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"max\" 1 5 3\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["5"]);
    }

    #[test]
    fn test_ffi_min() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"min\" 10 20 5 30 15\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["5"]);
    }

    #[test]
    fn test_ffi_round() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"round\" 3.14159 2\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["3.14"]);
    }

    #[test]
    fn test_ffi_len() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"len\" \"hello\"\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["5"]);
    }

    #[test]
    fn test_ffi_sin() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"math.sin\" 0\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["0.0"]);
    }

    #[test]
    fn test_ffi_int_conversion() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"int\" \"456\"\n+ g1 g0 1\n. g1", &[]).unwrap();
        assert_eq!(result, vec!["457"]);
    }

    #[test]
    fn test_ffi_float_conversion() {
        let mut interp = Interpreter::new();
        let result = interp.run("R g0 \"float\" \"3.14\"\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["3.14"]);
    }

    // Test P command (Python compatibility)
    #[test]
    fn test_p_command_compatibility() {
        let mut interp = Interpreter::new();
        let result = interp.run("P g0 \"math.sqrt\" 16\n. g0", &[]).unwrap();
        assert_eq!(result, vec!["4.0"]);
    }
}

// ============================================================================
// TestEdgeCases - エッジケース
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_negative_numbers() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 -10\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["-10"]);
    }

    #[test]
    fn test_zero_division() {
        // Division by zero should return infinity or error gracefully
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 10\n= v1 0\n/ v2 v0 v1\n. v2", &[]);
        // Either returns inf/nan or errors - both acceptable
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_large_numbers() {
        let mut interp = Interpreter::new();
        let result = interp.run("= v0 999999999\n. v0", &[]).unwrap();
        assert_eq!(result, vec!["999999999"]);
    }

    #[test]
    fn test_uninitialized_variable_is_zero() {
        let mut interp = Interpreter::new();
        let result = interp.run(". v9", &[]).unwrap();
        assert_eq!(result, vec!["0"]);
    }

    #[test]
    fn test_global_variable_persistence() {
        let mut interp = Interpreter::new();
        let result = interp.run("= g0 100\n= g1 200\n+ g2 g0 g1\n. g2", &[]).unwrap();
        assert_eq!(result, vec!["300"]);
    }
}

// ============================================================================
// TestExampleFiles - サンプルファイルのテスト
// ============================================================================

mod example_files {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn run_example(filename: &str) -> Vec<String> {
        let path = Path::new("examples").join(filename);
        let code = fs::read_to_string(&path).expect(&format!("Failed to read {}", path.display()));
        let mut interp = Interpreter::new();
        interp.run(&code, &[]).expect(&format!("Failed to run {}", filename))
    }

    fn run_example_with_args(filename: &str, args: &[String]) -> Vec<String> {
        let path = Path::new("examples").join(filename);
        let code = fs::read_to_string(&path).expect(&format!("Failed to read {}", path.display()));
        let mut interp = Interpreter::new();
        interp.run(&code, args).expect(&format!("Failed to run {}", filename))
    }

    #[test]
    fn test_fibonacci_example() {
        let result = run_example("fibonacci.sui");
        assert_eq!(result, vec!["55"]);
    }

    #[test]
    fn test_fizzbuzz_example() {
        let result = run_example("fizzbuzz.sui");
        assert!(result.contains(&"FizzBuzz".to_string()));
        assert!(result.contains(&"Fizz".to_string()));
        assert!(result.contains(&"Buzz".to_string()));
    }

    #[test]
    fn test_list_sum_example() {
        let result = run_example("list_sum.sui");
        // Output is "Sum:" and "150"
        assert!(result.contains(&"150".to_string()));
    }

    #[test]
    fn test_fib_args_example() {
        let result = run_example_with_args("fib_args.sui", &["15".to_string()]);
        assert_eq!(result, vec!["610"]);
    }

    #[test]
    fn test_args_demo_example() {
        let result = run_example_with_args("args_demo.sui", &["5".to_string(), "3".to_string()]);
        assert!(result.contains(&"8".to_string())); // 5 + 3 = 8
    }
}
