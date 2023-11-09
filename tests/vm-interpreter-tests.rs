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
fn fib_simple() {
    let output = interpret("tests/programs/fib_simple.zote");
    assert_eq!(output, "987\n");
}

#[test]
fn modify_captured() {
    let output = interpret("tests/programs/modify_captured.zote");
    assert_eq!(output, "1338\n1339\n1340\n1341\n");
}
