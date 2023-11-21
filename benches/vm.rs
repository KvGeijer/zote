#![feature(test)]

use test::Bencher;

extern crate test;

fn run_str(code: &str) {
    let stmts = parser::parse(code).expect("Was not able to parse code");
    let ast = semantic_analyzer::analyze_ast(&stmts);
    let res = vm::interpret_once(&ast);
    assert_eq!(res, 0);
}

#[bench]
fn vm_fibonachi(bench: &mut Bencher) {
    let code = include_str!("programs/fib.zote");
    bench.iter(|| run_str(code));
}

// #[bench]
// fn vm_string_manips(bench: &mut Bencher) {
//     let code = include_str!("programs/string_manips.zote");
//     bench.iter(|| run_str(code));
// }

// #[bench]
// fn aoc_2022_1(bench: &mut Bencher) {
//     let code = include_str!("../aoc-2022/day01.zote");
//     bench.iter(|| run_str(code));
// }

// #[bench]
// fn aoc_2022_2(bench: &mut Bencher) {
//     let code = include_str!("../aoc-2022/day02.zote");
//     bench.iter(|| run_str(code));
// }

// #[bench]
// fn aoc_2022_4(bench: &mut Bencher) {
//     let code = include_str!("../aoc-2022/day04.zote");
//     bench.iter(|| run_str(code));
// }
