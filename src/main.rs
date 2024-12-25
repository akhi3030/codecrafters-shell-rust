use std::io::{self, Write};

enum ContinueExec {
    Stop,
    Continue,
}

fn handle_command(input: &str) -> ContinueExec {
    if input == "exit 0" {
        return ContinueExec::Stop;
    }
    if input.starts_with("echo ") {
        let input = input.strip_prefix("echo ").unwrap();
        println!("{input}");
    } else {
        println!("{}: command not found", input);
    }
    ContinueExec::Continue
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();
        match handle_command(input) {
            ContinueExec::Continue => (),
            ContinueExec::Stop => break,
        }
    }
}
