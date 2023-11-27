use std::process::Command;

/// Interprets a file in 'aoc-2022/vm-solutions/{name}.zote
fn interpret_day(name: &str) -> String {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(format!("aoc-2022/vm-solutions/{name}.zote"))
        .output()
        .expect("Could not run file!");

    assert!(output.status.success(), "Could not run program!");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn vm_aoc_2022_1() {
    let output = interpret_day("day01");
    assert_eq!(output, "68923\n200044\n");
}

#[test]
fn vm_aoc_2022_2() {
    let output = interpret_day("day02");
    assert_eq!(output, "12586\n13193\n");
}
#[test]
fn vm_aoc_2022_3() {
    let output = interpret_day("day03");
    assert_eq!(output, "7568\n2780\n");
}
