#![feature(test)]

use test::Bencher;

extern crate test;

fn run_str(name: &str, code: &str) {
    let stmts = parser::parse(name, code).expect("Was not able to parse code");
    let ast = semantic_analyzer::analyze_ast(&stmts);
    let res = vm::interpret_once(&ast);
    assert_eq!(res, 0);
}

fn set_aoc_dir() {
    let _ = std::env::set_current_dir("aoc-2022/vm-solutions");
}

#[bench]
fn vm_fibonachi(bench: &mut Bencher) {
    let code = include_str!("programs/fib.zote");
    bench.iter(|| run_str("fib.zote", code));
}

#[bench]
fn vm_string_manips(bench: &mut Bencher) {
    let code = include_str!("programs/string_manips.zote");
    bench.iter(|| run_str("string_manips.zote", code));
}

#[bench]
fn vm_aoc_2022_1(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day01.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day0.zote", code));
}

#[bench]
fn vm_aoc_2022_2(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day02.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day1.zote", code));
}

#[bench]
fn vm_aoc_2022_3(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day03.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day2.zote", code));
}

#[bench]
fn vm_aoc_2022_4(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day04.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_5(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day05.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_6(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day06.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_7(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day07.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_8(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day08.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_9(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day09.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_10(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day10.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_11(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day11.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_12(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day12.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_13(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day13.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_14(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day14.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_15(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day15.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_16(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day16.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_17(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day17.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_18(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day18.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_19(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day19.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_20(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day20.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_21(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day21.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_22(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day22.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_23(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day23.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[ignore]
#[bench]
fn vm_aoc_2022_24(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day24.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}

#[bench]
fn vm_aoc_2022_25(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/vm-solutions/day25.zote");
    set_aoc_dir();
    bench.iter(|| run_str("day3.zote", code));
}
