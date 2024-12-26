use std::fs;
use std::io::{self, ErrorKind, Write};
use std::process::Command;

fn look_in_path(path: &[String], arg: &str) -> Option<String> {
    for dir in path {
        let dir = match fs::read_dir(dir) {
            Ok(dir) => dir,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => continue,
                _ => unimplemented!(),
            },
        };
        for file in dir {
            let file = file.unwrap();
            if file.file_name().to_str().unwrap() == arg {
                return Some(file.path().to_str().unwrap().to_string());
            }
        }
    }
    None
}

fn handle_type_command(path: &[String], arg: &str) {
    let cmd = parse_command(arg);
    match cmd {
        MyCmd::Builtin(_) => {
            println!("{} is a shell builtin", arg);
        }
        MyCmd::Other(_) => match look_in_path(path, arg) {
            Some(res) => println!("{} is {}", arg, res),
            None => println!("{}: not found", arg),
        },
    }
}

fn handle_other_command(path: &[String], argv0: String, argv: &[String]) {
    match look_in_path(path, &argv0) {
        Some(argv0) => {
            let res = Command::new(argv0).args(argv).output().unwrap();
            print!("{}", String::from_utf8_lossy(&res.stdout));
        }
        None => println!("{}: command not found", argv0),
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

enum MyCmd {
    Builtin(Builtin),
    Other(String),
}

fn parse_command(command_str: &str) -> MyCmd {
    if command_str == "exit" {
        MyCmd::Builtin(Builtin::Exit)
    } else if command_str == "echo" {
        MyCmd::Builtin(Builtin::Echo)
    } else if command_str == "type" {
        MyCmd::Builtin(Builtin::Type)
    } else {
        MyCmd::Other(command_str.to_string())
    }
}

fn parse_input(input: &str) -> (MyCmd, Vec<String>) {
    let mut input = input.split_whitespace();
    let command_str = input.next().unwrap();
    let command = parse_command(command_str);
    let rest = input.map(|s| s.to_string()).collect::<Vec<_>>();
    (command, rest)
}

fn handle_command(path: &[String], input: &str) -> ContinueExec {
    let (command, rest) = parse_input(input);
    match command {
        MyCmd::Builtin(built_in) => match built_in {
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
        MyCmd::Other(argv0) => {
            handle_other_command(path, argv0, &rest);
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
