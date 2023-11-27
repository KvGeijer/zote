use std::process::Command;

fn interpret(program: &str) -> String {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("ast-zote")
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
fn aoc_2022_1_ast() {
    let output = interpret("aoc-2022/ast-solutions/day01.zote");
    assert_eq!(output, "68923\n200044\n");
}

#[test]
fn aoc_2022_2_ast() {
    let output = interpret("aoc-2022/ast-solutions/day02.zote");
    assert_eq!(output, "12586\n13193\n");
}

#[test]
fn aoc_2022_3_ast() {
    let output = interpret("aoc-2022/ast-solutions/day03.zote");
    assert_eq!(output, "7568\n2780\n");
}

#[test]
fn aoc_2022_4_ast() {
    let output = interpret("aoc-2022/ast-solutions/day04.zote");
    assert_eq!(output, "584\n933\n");
}

#[test]
fn aoc_2022_5_ast() {
    let output = interpret("aoc-2022/ast-solutions/day05.zote");
    assert_eq!(output, "ZWHVFWQWW\nHZFZCCWWV\n");
}

#[test]
fn aoc_2022_6_ast() {
    let output = interpret("aoc-2022/ast-solutions/day06.zote");
    assert_eq!(output, "1723\n3708\n");
}

#[test]
fn aoc_2022_7_ast() {
    let output = interpret("aoc-2022/ast-solutions/day07.zote");
    assert_eq!(output, "1886043\n3842121\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_8_ast() {
    let output = interpret("aoc-2022/ast-solutions/day08.zote");
    assert_eq!(output, "1859\n332640\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_9_ast() {
    let output = interpret("aoc-2022/ast-solutions/day09.zote");
    assert_eq!(output, "6745\n2793\n");
}

#[test]
fn aoc_2022_10_ast() {
    let output = interpret("aoc-2022/ast-solutions/day10.zote");
    assert_eq!(output, "12540\n#### ####  ##  #### #### #    #  # #### \n#    #    #  #    # #    #    #  # #    \n###  ###  #      #  ###  #    #### ###  \n#    #    #     #   #    #    #  # #    \n#    #    #  # #    #    #    #  # #    \n#    ####  ##  #### #### #### #  # #### \n");
}

// A bit too slow to run all the times. Could be optimized.
#[ignore]
#[test]
fn aoc_2022_11_ast() {
    let output = interpret("aoc-2022/ast-solutions/day11.zote");
    assert_eq!(output, "120384\n32059801242\n");
}

#[test]
fn aoc_2022_12_ast() {
    let output = interpret("aoc-2022/ast-solutions/day12.zote");
    assert_eq!(output, "380\n375\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_13_ast() {
    let output = interpret("aoc-2022/ast-solutions/day13.zote");
    assert_eq!(output, "6369\n25800\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_14_ast() {
    let output = interpret("aoc-2022/ast-solutions/day14.zote");
    assert_eq!(output, "1078\n30157\n");
}

// The fact that this is incredibly slow already makes me scared for future solutions (day 16 and 19)
#[ignore]
#[test]
fn aoc_2022_15_ast() {
    let output = interpret("aoc-2022/ast-solutions/day15.zote");
    assert_eq!(output, "5525990\n11756174628223\n");
}

// Wow, this actually works quite fast!
#[ignore]
#[test]
fn aoc_2022_16_ast() {
    let output = interpret("aoc-2022/ast-solutions/day16.zote");
    assert_eq!(output, "1716\n2504\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_17_ast() {
    let output = interpret("aoc-2022/ast-solutions/day17.zote");
    assert_eq!(output, "3159\n1566272189352\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_18_ast() {
    let output = interpret("aoc-2022/ast-solutions/day18.zote");
    assert_eq!(output, "3448\n2052\n");
}

#[ignore]
#[test]
fn aoc_2022_19_ast() {
    let output = interpret("aoc-2022/ast-solutions/day19.zote");
    assert_eq!(output, "1356\n27720\n");
}

// Takes a couples seconds to run. On the edge whether to ignore or not
#[ignore]
#[test]
fn aoc_2022_20_ast() {
    let output = interpret("aoc-2022/ast-solutions/day20.zote");
    assert_eq!(output, "2827\n7834270093909\n");
}

#[test]
fn aoc_2022_21_ast() {
    let output = interpret("aoc-2022/ast-solutions/day21.zote");
    assert_eq!(output, "309248622142100\n3757272361782\n");
}

// Borderline slow
#[ignore]
#[test]
fn aoc_2022_22_ast() {
    let output = interpret("aoc-2022/ast-solutions/day22.zote");
    assert_eq!(output, "103224\n189097\n");
}

#[ignore]
#[test]
fn aoc_2022_23_ast() {
    let output = interpret("aoc-2022/ast-solutions/day23.zote");
    assert_eq!(output, "3871\n925\n");
}

#[ignore]
#[test]
fn aoc_2022_24_ast() {
    let output = interpret("aoc-2022/ast-solutions/day24.zote");
    assert_eq!(output, "299\n899\n");
}

#[test]
fn aoc_2022_25_ast() {
    let output = interpret("aoc-2022/ast-solutions/day25.zote");
    assert_eq!(output, "2--1=0=-210-1=00=-=1\n");
}
