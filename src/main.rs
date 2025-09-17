use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().starts_with("exit") {
                let exit_code = input.split_whitespace()
                    .nth(1)
                    .map(|s| s.parse::<i32>().unwrap_or_else(|_| {
                       1
                    }))
                    .unwrap_or(0);
                std::process::exit(exit_code);
        }

        println!("{}: command not found", input.trim().split_whitespace().next().unwrap());
    }
}
