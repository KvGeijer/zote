#![feature(test)]

use test::Bencher;

extern crate test;

fn run_str(code: &str) {
    let mut state = ast_interpreter::InterpreterState::new();
    let stmts = parser::parse(code).unwrap();
    ast_interpreter::interpret(&stmts, &mut state);
    assert!(!state.had_error());
}

#[bench]
fn fibonachi(bench: &mut Bencher) {
    let code = include_str!("programs/fib.zote");
    bench.iter(|| run_str(code));
}

#[bench]
fn prints(bench: &mut Bencher) {
    let code = include_str!("programs/prints.zote");
    bench.iter(|| run_str(code));
}

#[bench]
fn string_manips(bench: &mut Bencher) {
    let code = include_str!("programs/string_manips.zote");
    bench.iter(|| run_str(code));
}

#[bench]
fn aoc_2022_1(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/day01.zote");
    bench.iter(|| run_str(code));
}

#[bench]
fn aoc_2022_2(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/day02.zote");
    bench.iter(|| run_str(code));
}

#[bench]
fn aoc_2022_4(bench: &mut Bencher) {
    let code = include_str!("../aoc-2022/day04.zote");
    bench.iter(|| run_str(code));
}
