#![feature(test)]

use test::Bencher;
use zote::run_str;

extern crate test;

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
    let code = include_str!("../aoc-2022/inputs/02.txt");
    bench.iter(|| run_str(code));
}
