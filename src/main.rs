use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        let args = &parts[1..];
        match cmd {
            "exit" => {
                let exit_code = args.get(0)
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                std::process::exit(exit_code);
            },
            "echo" => {
                println!("{}", args.join(" "));
            },
            _ => println!("{}: command not found", input.trim().split_whitespace().next().unwrap()),

        }
    }
}
