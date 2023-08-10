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
        "1\n[1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765]\n6765\n"
    )
}

#[test]
fn caced_fibonachi() {
    let output = interpret("tests/programs/cached_fib.zote");
    assert_eq!(
        output,
        "[1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 0, 0, 0, 17711, 1, 5]\n"
    )
}

#[test]
fn pipes() {
    let output = interpret("tests/programs/pipes.zote");
    assert_eq!(
        output,
        "[1, 2, 3, 4, 5, 6, 7, 8, 9]\n[0, 1, 2, 3, 4, 5, 6, 7, 8]\n[0, 1, 2, 3, 4, 5, 6, 7, 8]\n"
    )
}

#[test]
fn sort() {
    let output = interpret("tests/programs/sort.zote");
    assert_eq!(output, "false\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\n")
}

#[test]
fn aoc_2022_1() {
    let output = interpret("aoc-2022/day01.zote");
    assert_eq!(output, "68923\n200044\n");
}

#[test]
fn aoc_2022_2() {
    let output = interpret("aoc-2022/day02.zote");
    assert_eq!(output, "12586\n13193\n");
}

#[test]
fn aoc_2022_3() {
    let output = interpret("aoc-2022/day03.zote");
    assert_eq!(output, "7568\n2780\n");
}

#[test]
fn aoc_2022_4() {
    let output = interpret("aoc-2022/day04.zote");
    assert_eq!(output, "584\n933\n");
}

#[test]
fn aoc_2022_5() {
    let output = interpret("aoc-2022/day05.zote");
    assert_eq!(output, "ZWHVFWQWW\nHZFZCCWWV\n");
}

#[test]
fn aoc_2022_6() {
    let output = interpret("aoc-2022/day06.zote");
    assert_eq!(output, "1723\n3708\n");
}

#[test]
fn aoc_2022_7() {
    let output = interpret("aoc-2022/day07.zote");
    assert_eq!(output, "1886043\n3842121\n");
}

#[test]
fn aoc_2022_8() {
    let output = interpret("aoc-2022/day08.zote");
    assert_eq!(output, "1859\n332640\n");
}

#[test]
fn aoc_2022_9() {
    let output = interpret("aoc-2022/day09.zote");
    assert_eq!(output, "6745\n2793\n");
}

#[test]
fn aoc_2022_10() {
    let output = interpret("aoc-2022/day10.zote");
    assert_eq!(output, "12540\n#### ####  ##  #### #### #    #  # #### \n#    #    #  #    # #    #    #  # #    \n###  ###  #      #  ###  #    #### ###  \n#    #    #     #   #    #    #  # #    \n#    #    #  # #    #    #    #  # #    \n#    ####  ##  #### #### #### #  # #### \n");
}

// A bit too slow to run all the times. Could be optimized.
#[ignore]
#[test]
fn aoc_2022_11() {
    let output = interpret("aoc-2022/day11.zote");
    assert_eq!(output, "120384\n32059801242\n");
}

#[test]
fn aoc_2022_12() {
    let output = interpret("aoc-2022/day12.zote");
    assert_eq!(output, "380\n375\n");
}

#[test]
fn aoc_2022_13() {
    let output = interpret("aoc-2022/day13.zote");
    assert_eq!(output, "6369\n25800\n");
}

#[test]
fn aoc_2022_14() {
    let output = interpret("aoc-2022/day14.zote");
    assert_eq!(output, "1078\n30157\n");
}

// The fact that this is incredibly slow already makes me scared for future solutions (day 16 and 19)
#[ignore]
#[test]
fn aoc_2022_15() {
    let output = interpret("aoc-2022/day15.zote");
    assert_eq!(output, "5525990\n11756174628223\n");
}

// Wow, this actually works quite fast!
#[ignore]
#[test]
fn aoc_2022_16() {
    let output = interpret("aoc-2022/day16.zote");
    assert_eq!(output, "1716\n2504\n");
}

#[test]
fn aoc_2022_17() {
    let output = interpret("aoc-2022/day17.zote");
    assert_eq!(output, "3159\n1566272189352\n");
}

#[test]
fn aoc_2022_18() {
    let output = interpret("aoc-2022/day18.zote");
    assert_eq!(output, "3448\n2052\n");
}

#[ignore]
#[test]
fn aoc_2022_19() {
    let output = interpret("aoc-2022/day19.zote");
    assert_eq!(output, "1356\n27720\n");
}

// Takes a couples seconds to run. On the edge whether to ignore or not
#[ignore]
#[test]
fn aoc_2022_20() {
    let output = interpret("aoc-2022/day20.zote");
    assert_eq!(output, "2827\n7834270093909\n");
}

#[test]
fn aoc_2022_21() {
    let output = interpret("aoc-2022/day21.zote");
    assert_eq!(output, "309248622142100\n3757272361782\n");
}
