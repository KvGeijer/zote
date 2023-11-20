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

fn interpret_error(program: &str) -> String {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(program)
        .output()
        .expect("Could not run file!");

    assert!(!output.status.success(), "Could run program!");
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn fib_simple() {
    let output = interpret("tests/programs/fib_simple.zote");
    assert_eq!(output, "987\n");
}

#[test]
fn fib() {
    let output = interpret("tests/programs/fib.zote");
    assert_eq!(
        output,
        "1\n[1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765]\n6765\n"
    )
}

#[test]
fn modify_captured() {
    let output = interpret("tests/programs/modify_captured.zote");
    assert_eq!(output, "1338\n1339\n1340\n1341\n");
}

#[test]
fn list_creation() {
    let output = interpret("tests/programs/list_creation.zote");
    assert_eq!(output, "[1, 2, 3, 4, 5]\n[1, 2, 3, 4, 5]\n[3, 2, 1, 0, -1, -2, -3]\n[5, 4, 3, 2, 1]\n[]\n[]\n[]\n[]\n[-5, -2, 1, 4]\n[0, 3, 6]\n[9, 6, 3]\n");
}

#[test]
fn list_slice_reading() {
    let output = interpret("tests/programs/list_slice_reading.zote");
    assert_eq!(output,"[1, 2, 3, 4, 5]\n[1, 2, 3, 4, 5]\n[1, 2, 3, 4, 5]\n[5, 4, 3, 2, 1]\n[5, 4, 3, 2, 1]\n[5, 4, 3, 2, 1]\n[5, 4, 3, 2]\n[5, 4, 3]\n[]\n[]\n[]\n[]\n[]\n[1]\n[1]\n[1]\n[]\n")
}

#[test]
fn list_index_reading() {
    let output = interpret("tests/programs/list_index_reading.zote");
    assert_eq!(
        output,
        "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n"
    );
}

#[test]
fn list_index_assigning() {
    let output = interpret("tests/programs/list_index_assigning.zote");
    assert_eq!(
        output,
        "[1, 1, 2, 3, 4]\n10\n[1, 10, 2, 3, 4]\n[101, 10, 2, 3, 4]\n[101, 10, 2, 3, -1]\n[101, 10, 2, -2, -1]\n"
    );
}

#[test]
fn list_functions() {
    let output = interpret("tests/programs/list_functions.zote");
    assert_eq!(output, "[1]\n[1, 2]\n2\n1\n[]\n");
}

#[test]
fn calls() {
    let output = interpret("tests/programs/calls.zote");
    assert_eq!(output, "1\n2\n3\n\n3\n4\n4\n\n1\n2\n4\n\n1\n2\n4\n");
}

#[test]
fn for_simple() {
    let output = interpret("tests/programs/for_simple.zote");
    assert_eq!(output, "1\n2\n3\n4\n5\n6\n7\n8\n9\n");
}

#[test]
fn list_slice_assignment() {
    let output = interpret("tests/programs/list_slice_assignment.zote");
    assert_eq!(
        output,
        "[1, 2, 3, 4, 5, 6]\n[0, 1, 2, 3, 5, 6]\n[11, 12, 13, 14, 15, 16]\n[0, 12, 0, 14, 0, 16]\n"
    )
}

#[test]
fn error_trace() {
    let output = interpret_error("tests/programs/error_trace.zote");
    assert!(output.contains("RUNTIME ERROR"));
    assert!(output.contains("out of bound"));
    assert!(output.contains("line 4"));
    assert!(output.contains("in f1"));
    assert!(output.contains("in f2"));
    assert!(output.contains("line 10"));
}

#[test]
fn assign_constant_ok() {
    let output = interpret("tests/programs/assign_constant_ok.zote");
    assert_eq!(output, "1\n2\n");
}

#[test]
fn assign_constant_error() {
    let output = interpret_error("tests/programs/assign_constant_error.zote");
    assert!(output.contains("to constant failed"));
}
