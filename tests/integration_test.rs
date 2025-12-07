//! Integration tests for Sui language

use sui_lang::interpreter::Interpreter;
use sui_lang::transpiler::{Sui2Py, Sui2Js};

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
= g0 10
$ g1 0 g0
. g1
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["55"]);
}

#[test]
fn test_fizzbuzz() {
    let code = r#"
= v0 1
: 0
> v1 v0 15
? v1 9
% v2 v0 15
~ v3 v2 0
? v3 1
% v4 v0 3
~ v5 v4 0
? v5 2
% v6 v0 5
~ v7 v6 0
? v7 3
. v0
@ 4
: 1
. "FizzBuzz"
@ 4
: 2
. "Fizz"
@ 4
: 3
. "Buzz"
@ 4
: 4
+ v0 v0 1
@ 0
: 9
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();

    // First 15 outputs
    assert_eq!(output[0], "1");
    assert_eq!(output[1], "2");
    assert_eq!(output[2], "Fizz");
    assert_eq!(output[3], "4");
    assert_eq!(output[4], "Buzz");
    assert_eq!(output[5], "Fizz");
    assert_eq!(output[14], "FizzBuzz");
}

#[test]
fn test_command_line_args() {
    let code = r#"
. g100
. g101
. g102
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &["hello".to_string(), "42".to_string()]).unwrap();
    assert_eq!(output, vec!["2", "hello", "42"]);
}

#[test]
fn test_array_operations() {
    let code = r#"
[ v0 5
{ v0 0 10
{ v0 1 20
{ v0 2 30
] v1 v0 0
] v2 v0 1
] v3 v0 2
+ v4 v1 v2
+ v5 v4 v3
. v5
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["60"]);
}

#[test]
fn test_string_output() {
    let code = r#"
. "Hello, World!"
. "Line 2"
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["Hello, World!", "Line 2"]);
}

#[test]
fn test_nested_functions() {
    let code = r#"
# 0 1 {
+ v0 a0 1
^ v0
}
# 1 1 {
$ v0 0 a0
$ v1 0 v0
^ v1
}
$ g0 1 5
. g0
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["7"]); // 5 + 1 + 1 = 7
}

#[test]
fn test_comparison_operations() {
    let code = r#"
= v0 5
= v1 10
< v2 v0 v1
> v3 v0 v1
~ v4 v0 v0
. v2
. v3
. v4
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["1", "0", "1"]);
}

#[test]
fn test_logical_operations() {
    let code = r#"
= v0 1
= v1 0
& v2 v0 v0
& v3 v0 v1
| v4 v0 v1
! v5 v0
! v6 v1
. v2
. v3
. v4
. v5
. v6
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["1", "0", "1", "0", "1"]);
}

#[test]
fn test_sui2py_simple() {
    let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;

    let mut transpiler = Sui2Py::new();
    let python_code = transpiler.transpile_to_python(code).unwrap();

    assert!(python_code.contains("v0 = 10"));
    assert!(python_code.contains("v1 = v0 + 5"));
    assert!(python_code.contains("print(v1)"));
}

#[test]
fn test_sui2js_simple() {
    let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;

    let mut transpiler = Sui2Js::new();
    let js_code = transpiler.transpile_to_js(code).unwrap();

    assert!(js_code.contains("v0 = 10;"));
    assert!(js_code.contains("v1 = v0 + 5;"));
    assert!(js_code.contains("console.log(v1);"));
}

#[test]
fn test_float_arithmetic() {
    let code = r#"
= v0 3.14
= v1 2.0
* v2 v0 v1
. v2
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert!(output[0].starts_with("6.28"));
}

#[test]
fn test_modulo() {
    let code = r#"
= v0 17
= v1 5
% v2 v0 v1
. v2
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_simple_loop() {
    let code = r#"
= v0 0
= v1 0
: 0
< v2 v0 5
! v3 v2
? v3 1
+ v1 v1 v0
+ v0 v0 1
@ 0
: 1
. v1
"#;

    let mut interp = Interpreter::new();
    let output = interp.run(code, &[]).unwrap();
    assert_eq!(output, vec!["10"]); // 0+1+2+3+4 = 10
}
