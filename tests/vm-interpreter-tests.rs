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
