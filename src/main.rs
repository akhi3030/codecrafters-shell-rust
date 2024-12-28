use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::process::Command;

fn split_string(input: String) -> Vec<String> {
    let mut ret = vec![];
    let mut current = String::new();
    let mut single_quote = false;
    let mut double_quote = false;
    let mut slash = false;
    for c in input.chars() {
        match (single_quote, double_quote, c) {
            (true, true, _) => unimplemented!(),
            (false, false, c) => {
                if slash {
                    current.push(c);
                    slash = false;
                    continue;
                }
                if c == '\\' {
                    slash = true;
                    continue;
                }
                if c == '\'' {
                    single_quote = true;
                    continue;
                }
                if c == '"' {
                    double_quote = true;
                    continue;
                }
                if c == ' ' {
                    if current.len() != 0 {
                        ret.push(current);
                        current = String::new();
                    }
                    continue;
                }
                current.push(c);
            }

            (true, false, c) => {
                if c == '\'' {
                    single_quote = false;
                    continue;
                }
                current.push(c);
            }
            (false, true, c) => {
                if slash {
                    if c == '\\' || c == '$' || c == '"' {
                        current.push(c);
                    } else {
                        current.push('\\');
                        current.push(c);
                    }
                    slash = false;
                    continue;
                }
                if c == '\\' {
                    slash = true;
                    continue;
                }
                if c == '"' {
                    double_quote = false;
                    continue;
                }
                current.push(c);
            }
        }
    }
    if current.len() != 0 {
        ret.push(current);
    }
    ret
}

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

fn handle_cd_command(mut stderr: Box<dyn io::Write>, mut args: Vec<String>) {
    assert_eq!(args.len(), 1);
    let path = args.pop().unwrap();
    let path = if path == "~" {
        std::env::var("HOME").unwrap()
    } else {
        path
    };
    match std::env::set_current_dir(&path) {
        Ok(()) => (),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                writeln!(stderr, "cd: {}: No such file or directory", path).unwrap();
            }
            _ => unimplemented!(),
        },
    }
}

fn handle_echo_command(mut stdout: Box<dyn io::Write>, args: Vec<String>) {
    writeln!(stdout, "{}", args.join(" ")).unwrap();
}

fn handle_pwd_command(mut stdout: Box<dyn io::Write>) {
    writeln!(
        stdout,
        "{}",
        std::env::current_dir().unwrap().to_str().unwrap()
    )
    .unwrap();
}

fn handle_type_command(
    mut stdout: Box<dyn io::Write>,
    mut stderr: Box<dyn io::Write>,
    path: &[String],
    arg: &str,
) {
    let cmd = parse_command(arg);
    match cmd {
        MyCmd::Builtin(_) => {
            writeln!(stdout, "{} is a shell builtin", arg).unwrap();
        }
        MyCmd::Other(_) => match look_in_path(path, arg) {
            Some(res) => writeln!(stdout, "{} is {}", arg, res).unwrap(),
            None => writeln!(stderr, "{}: not found", arg).unwrap(),
        },
    }
}

fn handle_other_command(
    mut stdout: Box<dyn io::Write>,
    mut stderr: Box<dyn io::Write>,
    path: &[String],
    argv0: String,
    argv: &[String],
) {
    match look_in_path(path, &argv0) {
        Some(argv0) => {
            let res = Command::new(argv0).args(argv).output().unwrap();
            let out = String::from_utf8_lossy(&res.stdout);
            let err = String::from_utf8_lossy(&res.stderr);
            write!(stdout, "{}", out).unwrap();
            write!(stderr, "{}", err).unwrap();
        }
        None => writeln!(stdout, "{}: command not found", argv0).unwrap(),
    }
}

enum ContinueExec {
    Stop,
    Continue,
}

enum Builtin {
    Cd,
    Echo,
    Exit,
    Pwd,
    Type,
}

enum MyCmd {
    Builtin(Builtin),
    Other(String),
}

fn parse_command(command_str: &str) -> MyCmd {
    if command_str == "cd" {
        MyCmd::Builtin(Builtin::Cd)
    } else if command_str == "exit" {
        MyCmd::Builtin(Builtin::Exit)
    } else if command_str == "echo" {
        MyCmd::Builtin(Builtin::Echo)
    } else if command_str == "pwd" {
        MyCmd::Builtin(Builtin::Pwd)
    } else if command_str == "type" {
        MyCmd::Builtin(Builtin::Type)
    } else {
        MyCmd::Other(command_str.to_string())
    }
}

fn setup_output(input: Vec<String>) -> (Box<dyn io::Write>, Box<dyn io::Write>, Vec<String>) {
    let mut stdout: Box<dyn io::Write> = Box::new(io::stdout());
    let mut stderr: Box<dyn io::Write> = Box::new(io::stderr());
    let mut ret = vec![];
    let mut change_stdout = false;
    let mut change_stderr = false;
    for item in input {
        if change_stdout {
            stdout = Box::new(File::create(item).unwrap());
            change_stdout = false;
            continue;
        }
        if change_stderr {
            stderr = Box::new(File::create(item).unwrap());
            change_stderr = false;
            continue;
        }
        if item == ">" || item == "1>" {
            change_stdout = true;
            continue;
        }
        if item == "2>" {
            change_stderr = true;
            continue;
        }
        ret.push(item);
    }
    (stdout, stderr, ret)
}

fn parse_input(input: String) -> (Box<dyn io::Write>, Box<dyn io::Write>, MyCmd, Vec<String>) {
    let input = split_string(input);
    let (stdout, stderr, mut input) = setup_output(input);
    let command_str = input.remove(0);
    let command = parse_command(&command_str);
    let rest = input.into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
    (stdout, stderr, command, rest)
}

fn handle_command(path: &[String], input: String) -> ContinueExec {
    let (stdout, stderr, command, rest) = parse_input(input);
    match command {
        MyCmd::Builtin(builtin) => match builtin {
            Builtin::Cd => {
                handle_cd_command(stderr, rest);
                ContinueExec::Continue
            }
            Builtin::Echo => {
                handle_echo_command(stdout, rest);
                ContinueExec::Continue
            }
            Builtin::Exit => {
                assert_eq!(rest.len(), 1);
                assert_eq!(rest[0], "0");
                ContinueExec::Stop
            }
            Builtin::Pwd => {
                handle_pwd_command(stdout);
                ContinueExec::Continue
            }
            Builtin::Type => {
                assert_eq!(rest.len(), 1);
                handle_type_command(stdout, stderr, path, &rest[0]);
                ContinueExec::Continue
            }
        },
        MyCmd::Other(argv0) => {
            handle_other_command(stdout, stderr, path, argv0, &rest);
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
        let input = input.trim().to_string();
        match handle_command(&path, input) {
            ContinueExec::Continue => (),
            ContinueExec::Stop => break,
        }
    }
}
