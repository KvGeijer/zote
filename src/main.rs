use clap::Parser;
use std::fs;
use std::io::{stdin, stdout, Write};

mod errors;
mod interpreter;
mod parser;
mod scanner;

#[derive(Parser)]
struct Args {
    /// The script to run
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(ref file) = args.file {
        run_file(file);
    } else {
        run_repl();
    }
}

// Maybe move these out to separate file for running and keeping state
fn run_file(file: &str) {
    let script = fs::read_to_string(file).expect("Could not open file.");

    run(&script, &mut errors::ErrorReporter::new());
}

fn run_repl() {
    let reader = stdin();
    let mut line = String::new();
    let mut error_reporter = errors::ErrorReporter::new();

    println!("Running!!!");

    while {
        print!("> ");
        stdout().flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap_or(0) > 0
    } {
        println!(" REPL got {:?}", line);
        // Does not preserve program state between calls
        run(&line, &mut error_reporter);
        error_reporter.reset();
    }
}
fn run(code: &str, error_reporter: &mut errors::ErrorReporter) {
    let tokens = scanner::tokenize(code, error_reporter);

    // for token in tokens.iter() {
    //     println!("{:?}", token);
    // }

    let expr = parser::parse(&tokens, error_reporter);

    if error_reporter.has_error {
        return;
    }

    println!("Parsed expression: {:?}", expr);
}
