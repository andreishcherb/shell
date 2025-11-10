#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Command {
    Exit,
    Echo,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            _ => Err(ParseCommandError),
        }
    }
}

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

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
            },
            Err(_) => println!("{}: command not found", input[0]),
        }
    }
}
