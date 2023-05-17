use std::process::Command;

fn interpret(program: &str) -> String {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(program)
        .output()
        .expect("Could not run file!");

    assert!(output.status.success(), "Could not run program!");

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn fibonachi() {
    let output = interpret("tests/programs/fib.zote");
    assert_eq!(
        output,
        "[1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765]\n"
    )
}

// #[test]
// fn float_comparisons() {
//     // Not the prettiest, but easy to find if one fails, rather than having one big string.
//     let program = "2.5*3125.0 > 2.499*3125.0";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "0.0 == 0.0";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "2.2/5.1 - 3.5*5.0 < -17.0";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "!(1.1>=1.100001)";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "!(2.2 != 2.2)";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "1.1 <= 1.01*1.11";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "2.000000001 % 0.1 < 0.00001";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));

//     let program = "2.2^-2.2 >= 0.176";
//     run_str(program);
//     assert!(matches!(val, Value::Bool(true)));
// }
