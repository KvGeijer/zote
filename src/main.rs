use clap::Parser;
use std::{io, fs};

mod scanner;
mod errors;

#[derive(Parser)]
struct Args {
    /// The script to run
    file: Option<String>
}

pub struct MainState {
    has_error: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(ref file) = args.file {
        run_file(file);
    } else {
        run_repl();
    }
}

impl MainState {
    fn new() -> Self {
        Self { has_error: false }
    }

    fn reset(&mut self) {
        self.has_error = false;
    }
}

// Maybe move these out to separate file for running and keeping state
fn run_file(file: &str) {
    let script = fs::read_to_string(file)
        .expect("Could not open file.");

    run(&script, &mut MainState::new());
}

fn run_repl() {
    let reader = io::stdin();
    let mut line = String::new();
    let mut state = MainState::new();

    println!("Running!!!");

    while {
        println!("> ");
        reader.read_line(&mut line).unwrap_or(0) > 0
    } {
        println!("{:?}", line);
        run(&line, &mut state);
        state.reset();
    }
}
fn run(code: &str, state: &mut MainState) {
    let tokens = scanner::tokenize(code, state);

    for token in tokens {
        println!("{:?}", token);
    }
}

