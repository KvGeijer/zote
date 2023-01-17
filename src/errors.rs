// Should be expanded and changed when more is clear
pub fn error(line: usize, message: &str, has_error: &mut bool) {
    report(line, "", message, has_error);
}

fn report(line: usize, place: &str, message: &str, has_error: &mut bool) {
    eprintln!("[line {line}] Error{place}: {message}");
    *has_error = true;
}

