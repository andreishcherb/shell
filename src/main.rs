#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Command {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            "pwd"  => Ok(Self::Pwd),
            "cd"   => Ok(Self::Cd),
            
            _ => Err(ParseCommandError),
        }
    }
}

use std::fmt;

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Exit => write!(f, "exit"),
            Command::Echo => write!(f, "echo"),
            Command::Type => write!(f, "type"),
            Command::Pwd  => write!(f, "pwd"),
            Command::Cd   => write!(f, "cd"),
        }
    }
}

use std::path::Path;

fn main() -> std::io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input: Vec<&str> = input.trim().split(' ').collect();
        let cmd = input[0].parse::<Command>();
        match cmd {
            Ok(cmd) => match cmd {
                Command::Exit => std::process::exit(0),
                Command::Echo => {
                    let mut index = 1;
                    while index < input.len() {
                        print!("{}", input[index]);
                        if index != input.len() - 1 {
                            print!(" ")
                        }
                        index += 1;
                    }
                    println!()
                }
                Command::Pwd => println!("{}",env::current_dir()?.display()),
                Command::Cd => {
                    if input.len() > 1 {
                        let root = Path::new(input[1]);
                        if let Err(_) = env::set_current_dir(&root) {
                            println!("cd: {}: No such file or directory", input[1])
                        }
                    }
                }
                Command::Type => {
                    if input.len() > 1 {
                        let cmd = input[1].parse::<Command>();
                        match cmd {
                            Ok(cmd) => println!("{} is a shell builtin", cmd),
                            Err(_) => match search_executable_file(input[1], "PATH") {
                                Some(path) => println!("{} is {}", input[1], path.display()),
                                None => println!("{}: not found", input[1]),
                            },
                        }
                    }
                }
            },
            Err(_) => match search_executable_file(input[0], "PATH") {
                Some(path) => {
                    let mut cmd = std::process::Command::new(path.file_name().unwrap_or(path.as_os_str()));
                    let mut index = 1;
                    while index < input.len() {
                        cmd.arg(input[index]);
                        index += 1;
                    }
                    let output = cmd.output()?;
                    io::stdout().write_all(&output.stdout)?;
                    io::stderr().write_all(&output.stderr)?;
                }
                None => println!("{}: not found", input[0]),
            },
        }
    }
}

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn search_executable_file(filename: &str, key: &str) -> Option<PathBuf> {
    match env::var(key) {
        Ok(val) => {
            let dirs: Vec<&str> = val.split(':').collect();
            for dir in dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() && entry.file_name() == filename {
                                    if let Ok(metadata) = entry.metadata() {
                                        if metadata.permissions().mode() & 0o100 != 0 {
                                            return Some(entry.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        Err(e) => {
            println!("env var: {key} not found: {e}");
            None
        }
    }
}
