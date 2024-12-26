use std::fs;
use std::io::{self, Write};

fn handle_type_command(path: &[String], arg: &str) {
    let cmd = parse_command(arg);
    match cmd {
        Command::Builtin(_) => {
            println!("{} is a shell builtin", arg);
        }
        Command::Other => {
            for dir in path {
                let dir = fs::read_dir(dir).unwrap();
                for file in dir {
                    let file = file.unwrap();
                    if file.file_name().to_str().unwrap() == arg {
                        println!("{} is {}", arg, file.path().to_str().unwrap());
                        return;
                    }
                }
            }
            println!("{}: not found", arg);
        }
    }
}

enum ContinueExec {
    Stop,
    Continue,
}

enum Builtin {
    Echo,
    Exit,
    Type,
}

enum Command {
    Builtin(Builtin),
    Other,
}

fn parse_command(command_str: &str) -> Command {
    if command_str == "exit" {
        Command::Builtin(Builtin::Exit)
    } else if command_str == "echo" {
        Command::Builtin(Builtin::Echo)
    } else if command_str == "type" {
        Command::Builtin(Builtin::Type)
    } else {
        Command::Other
    }
}

fn parse_input(input: &str) -> (Command, Vec<String>) {
    let mut input = input.split_whitespace();
    let command_str = input.next().unwrap();
    let command = parse_command(command_str);
    let rest = input.map(|s| s.to_string()).collect::<Vec<_>>();
    (command, rest)
}

fn handle_command(path: &[String], input: &str) -> ContinueExec {
    let (command, rest) = parse_input(input);
    match command {
        Command::Builtin(built_in) => match built_in {
            Builtin::Echo => {
                println!("{}", rest.join(" "));
                ContinueExec::Continue
            }
            Builtin::Exit => {
                assert_eq!(rest.len(), 1);
                assert_eq!(rest[0], "0");
                ContinueExec::Stop
            }
            Builtin::Type => {
                assert_eq!(rest.len(), 1);
                handle_type_command(path, &rest[0]);
                ContinueExec::Continue
            }
        },
        Command::Other => {
            println!("{}: command not found", input);
            ContinueExec::Continue
        }
    }
}

fn main() {
    let path = std::env::var("PATH")
        .unwrap_or(String::new())
        .split(':')
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();
        match handle_command(&path, input) {
            ContinueExec::Continue => (),
            ContinueExec::Stop => break,
        }
    }
}
