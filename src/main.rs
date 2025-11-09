#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        if command.trim() == "exit" {
            std::process::exit(0);
        } 
        else {
            println!("{}: command not found", command.trim());
        }

    }
}
