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

#[test]
fn vm_aoc_2022_14() {
    let output = interpret_day("day14");
    assert_eq!(output, "1078\n30157\n");
}

// Still incredibly slow... Taking 2 minutes to run :s
#[ignore]
#[test]
fn vm_aoc_2022_15() {
    let output = interpret_day("day15");
    assert_eq!(output, "5525990\n11756174628223\n");
}

#[ignore]
#[test]
fn vm_aoc_2022_16() {
    let output = interpret_day("day16");
    assert_eq!(output, "1716\n2504\n");
}

#[test]
fn vm_aoc_2022_17() {
    let output = interpret_day("day17");
    assert_eq!(output, "3159\n1566272189352\n");
}

#[test]
fn vm_aoc_2022_18() {
    let output = interpret_day("day18");
    assert_eq!(output, "3448\n2052\n");
}

#[ignore]
#[test]
fn vm_aoc_2022_19() {
    let output = interpret_day("day19");
    assert_eq!(output, "1356\n27720\n");
}

#[ignore]
#[test]
fn vm_aoc_2022_20() {
    let output = interpret_day("day20");
    assert_eq!(output, "2827\n7834270093909\n");
}

#[test]
fn vm_aoc_2022_21() {
    let output = interpret_day("day21");
    assert_eq!(output, "309248622142100\n3757272361782\n");
}

#[test]
fn vm_aoc_2022_22() {
    let output = interpret_day("day22");
    assert_eq!(output, "103224\n189097\n");
}

#[ignore]
#[test]
fn vm_aoc_2022_23() {
    let output = interpret_day("day23");
    assert_eq!(output, "3871\n925\n");
}

#[ignore]
#[test]
fn vm_aoc_2022_24() {
    let output = interpret_day("day24");
    assert_eq!(output, "299\n899\n");
}

#[test]
fn vm_aoc_2022_25() {
    let output = interpret_day("day25");
    assert_eq!(output, "2--1=0=-210-1=00=-=1\n");
}
