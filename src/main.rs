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
            "pwd" => Ok(Self::Pwd),
            "cd" => Ok(Self::Cd),

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
            Command::Pwd => write!(f, "pwd"),
            Command::Cd => write!(f, "cd"),
        }
    }
}

use regex::Regex;
use std::fs::File;

fn main() -> std::io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim().replace("''", "").replace("\"\"", "");
        if input.is_empty() {
            continue;
        }

        let re = Regex::new(r#"("[/'\w\s\\]+"|'[^']+'|[~/\.\-\w\\\d>]+)"#).unwrap();
        let mut args = vec![];
        for (_, [arg]) in re.captures_iter(&input).map(|c| c.extract()) {
            let x: &[_] = &['\'', '"'];
            let arg = arg.trim_matches(x);
            args.push(arg);
        }

        let cmd = args[0].parse::<Command>();
        match cmd {
            Ok(cmd) => match cmd {
                Command::Exit => std::process::exit(0),
                Command::Echo => {
                    let redirect_op = (">", "1>");
                    if let Some(_) = args
                        .iter()
                        .find(|&&s| s == redirect_op.0 || s == redirect_op.1)
                    {
                        for (i, arg) in args.iter().enumerate() {
                            if *arg == redirect_op.0 || *arg == redirect_op.1 && i != args.len() - 1 {
                                let file_name = args[i + 1];
                                let mut file = File::create(file_name)?;
                                let mut index = 1;
                                while index < i {
                                    file.write_all(args[index].as_bytes())?;
                                    file.write_all(b" ")?;
                                    index += 1;
                                }
                                file.write_all(b"\n")?;
                                break;
                            } else if *arg == redirect_op.0 || *arg == redirect_op.1 && i == args.len() - 1 {
                                println!("rash: missing file name");
                            }
                        }
                    } else {
                        let mut index = 1;
                        while index < args.len() {
                            print!("{}", args[index]);
                            if index != args.len() - 1 {
                                print!(" ")
                            }
                            index += 1;
                        }
                        println!()
                    }
                }
                Command::Pwd => println!("{}", env::current_dir()?.display()),
                Command::Cd => {
                    if args.len() > 1 {
                        let parsed_directory =
                            args[1].replace("~", &std::env::var("HOME").unwrap());
                        if let Err(_) = env::set_current_dir(parsed_directory) {
                            println!("cd: {}: No such file or directory", args[1])
                        }
                    }
                }
                Command::Type => {
                    if args.len() > 1 {
                        let cmd = args[1].parse::<Command>();
                        match cmd {
                            Ok(cmd) => println!("{} is a shell builtin", cmd),
                            Err(_) => match search_executable_file(args[1], "PATH") {
                                Some(path) => println!("{} is {}", args[1], path.display()),
                                None => println!("{}: not found", args[1]),
                            },
                        }
                    }
                }
            },
            Err(_) => match search_executable_file(args[0], "PATH") {
                Some(path) => {
                    let mut cmd =
                        std::process::Command::new(path.file_name().unwrap_or(path.as_os_str()));
                    let mut index = 1;
                    while index < args.len() {
                        cmd.arg(args[index]);
                        index += 1;
                    }
                    let output = cmd.output()?;
                    io::stdout().write_all(&output.stdout)?;
                    io::stderr().write_all(&output.stderr)?;
                }
                None => println!("{}: not found", args[0]),
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
