use std::io::{self, Write};

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
    Other(String),
}

fn parse_command(command_str: &str) -> Command {
    if command_str == "exit" {
        Command::Builtin(Builtin::Exit)
    } else if command_str == "echo" {
        Command::Builtin(Builtin::Echo)
    } else if command_str == "type" {
        Command::Builtin(Builtin::Type)
    } else {
        Command::Other(command_str.to_string())
    }
}

fn parse_input(input: &str) -> (Command, Vec<String>) {
    let mut input = input.split_whitespace();
    let command_str = input.next().unwrap();
    let command = parse_command(command_str);
    let rest = input.map(|s| s.to_string()).collect::<Vec<_>>();
    (command, rest)
}

fn handle_command(input: &str) -> ContinueExec {
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
                let cmd = parse_command(&rest[0]);
                match cmd {
                    Command::Builtin(_) => {
                        println!("{} is a shell builtin", &rest[0]);
                    }
                    Command::Other(_) => {
                        println!("{}: not found", &rest[0]);
                    }
                }
                ContinueExec::Continue
            }
        },
        Command::Other(cmd) => {
            println!("{}: command not found", cmd);
            ContinueExec::Continue
        }
    }
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
