#![feature(test)]

use std::process::Command;

use test::Bencher;

extern crate test;

fn interpret(program: &str) -> String {
    let output = Command::new("./target/release/zote")
        .arg(program)
        .output()
        .expect("Could not run command!");

    assert!(output.status.success(), "Could not run program!");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn build() {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .output()
        .expect("Could not build interpreter!");
    assert!(
        output.status.success(),
        "Could not successfully build interpreter!"
    );
}

#[bench]
fn fibonachi(bench: &mut Bencher) {
    build();
    bench.iter(|| interpret("benches/programs/fib.zote"));
}

#[bench]
fn prints(bench: &mut Bencher) {
    build();
    bench.iter(|| interpret("benches/programs/prints.zote"));
}

#[bench]
fn string_manips(bench: &mut Bencher) {
    build();
    bench.iter(|| interpret("benches/programs/string_manips.zote"));
}
