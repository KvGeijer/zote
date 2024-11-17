use clap::Parser;
use std::{
    fs,
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    process::exit,
};
use vm::value::get_natives;

#[derive(Parser)]
struct Args {
    /// The script to run
    file: Option<String>,

    /// Format the parsed code instead of running it when set
    #[clap(short, long, requires = "file", conflicts_with = "doc_functions")]
    format: bool,

    /// Output all the globally declared functions in the file
    #[clap(short, long, conflicts_with = "format")]
    doc_functions: bool,
}

fn main() {
    let args = Args::parse();

    if args.doc_functions {
        document_functions(args.file);
    } else if let Some(ref file) = args.file {
        if args.format {
            format_file(file);
        } else {
            exit(run_file(file));
        }
    } else {
        run_repl();
    }
}

/// Interprets a text file as a Zote script, returning the exit code.
fn run_file(file: &str) -> i32 {
    let script = fs::read_to_string(file).expect("Could not open file.");
    let saved = change_dir(file);
    let res = run_str(file, &script);
    restore_dir(saved);
    res
}

/// Parses the code in a file, outputting a formatted version of the code
fn format_file(file: &str) {
    let script = fs::read_to_string(file).expect("Could not open file.");
    change_dir(file);
    if let Some(stmts) = parser::parse(file, &script) {
        println!("{}", semantic_analyzer::format_parsed(&stmts));
    } else {
        exit(65)
    }
}

fn document_functions(file_path: Option<String>) {
    // First the builtin functions
    let mut docs = String::new();
    for native_str in get_natives().iter().map(|native| native.debug_print()) {
        docs.push_str(&format!("fn {native_str};\n"));
    }

    // Then the ones from the potential file
    if let Some(ref file) = file_path {
        let script = fs::read_to_string(file).expect("Could not open file.");
        change_dir(file);
        if let Some(stmts) = parser::parse(file, &script) {
            docs.push_str(&parser::gen_functions_doc(&stmts))
        } else {
            exit(65)
        }
    }

    println!("{docs}");
}

/// Change the working dir to the files dir, and then return the previous dir
fn change_dir(file_path: &str) -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    std::env::set_current_dir(Path::new(file_path).parent()?).ok()?;
    Some(current_dir)
}

/// Restores the working dir if it was succesfully changed
fn restore_dir(saved: Option<PathBuf>) {
    if let Some(path) = saved {
        std::env::set_current_dir(&path).expect("Was not able to change back the path!");
    }
}

/// Interprets the string as if from a file.
fn run_str(name: &str, code: &str) -> i32 {
    if let Some(stmts) = parser::parse(name, &code) {
        let ast = semantic_analyzer::analyze_ast(&stmts);
        vm::interpret_once(&ast)
    } else {
        65
    }
}

/// Starts the Zote repl.
///
/// Each line must be a statement or an expression (in which case its value is printed),
/// and the state of the program is preserved between lines.
fn run_repl() {
    let reader = stdin();
    let mut line = String::new();
    // TODO: Save state between repl calls

    while {
        print!("> ");
        stdout().flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap_or(0) > 0
    } {
        // Does not preserve program state between calls
        run_str("REPL", &line);
    }
}
