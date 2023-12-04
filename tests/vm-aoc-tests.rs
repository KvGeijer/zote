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

#[test]
fn vm_aoc_2022_4() {
    let output = interpret_day("day04");
    assert_eq!(output, "584\n933\n");
}

#[test]
fn vm_aoc_2022_5() {
    let output = interpret_day("day05");
    assert_eq!(output, "ZWHVFWQWW\nHZFZCCWWV\n");
}

#[test]
fn vm_aoc_2022_6() {
    let output = interpret_day("day06");
    assert_eq!(output, "1723\n3708\n");
}

#[test]
fn vm_aoc_2022_7() {
    let output = interpret_day("day07");
    assert_eq!(output, "1886043\n3842121\n");
}

#[test]
fn vm_aoc_2022_8() {
    let output = interpret_day("day08");
    assert_eq!(output, "1859\n332640\n");
}

#[test]
fn vm_aoc_2022_9() {
    let output = interpret_day("day09");
    assert_eq!(output, "6745\n2793\n");
}

#[test]
fn vm_aoc_2022_10() {
    let output = interpret_day("day10");
    assert_eq!(output, "12540\n#### ####  ##  #### #### #    #  # #### \n#    #    #  #    # #    #    #  # #    \n###  ###  #      #  ###  #    #### ###  \n#    #    #     #   #    #    #  # #    \n#    #    #  # #    #    #    #  # #    \n#    ####  ##  #### #### #### #  # #### \n");
}

#[ignore]
#[test]
fn vm_aoc_2022_11() {
    let output = interpret_day("day11");
    assert_eq!(output, "120384\n32059801242\n");
}

#[test]
fn vm_aoc_2022_12() {
    let output = interpret_day("day12");
    assert_eq!(output, "380\n375\n");
}

#[test]
fn vm_aoc_2022_13() {
    let output = interpret_day("day13");
    assert_eq!(output, "6369\n25800\n");
}
