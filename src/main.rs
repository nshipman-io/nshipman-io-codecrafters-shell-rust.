use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};
use is_executable::IsExecutable;

enum Command {
    Exit(Option<i32>),
    Echo(Vec<String>),
    Type(String),
}

struct Builtin {
    name: &'static str,
}

impl Command {
    const BUILTINS: &'static [Builtin] = &[
        Builtin { name: "exit", },
        Builtin { name: "echo", },
        Builtin { name: "type", }
    ];

    fn is_builtin(name: &str) -> bool {
        Self::BUILTINS.iter().any(|b| b.name == name)
    }

    fn parse(input: &str) -> Result<Command, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Err("".to_string());
        }

        let cmd = parts[0];
        let args = &parts[1..];
        match cmd {
            "exit" => {
                let code = args.get(0)
                    .and_then(|s| s.parse::<i32>().ok());
                Ok(Command::Exit(code))
            },
            "echo" => {
                let args = args.iter().map(|s| s.to_string())
                    .collect();
                Ok(Command::Echo(args))
            },
            "type" => {
                match args.get(0) {
                    Some(cmd) => Ok(Command::Type(cmd.to_string())),
                    None => Err("type: missing argument".to_string())
                }
            }
            _ => Err(format!("{}: command not found", cmd))
        }
    }

    fn execute(self) {
        match self {
            Command::Exit(code) => {
                std::process::exit(code.unwrap_or(0));
            }
            Command::Echo(args) => {
                println!("{}", args.join(" "));
            }

            Command::Type(cmd) => {
                if Self::is_builtin(&cmd) {
                   println!("{} is a shell builtin", cmd);
                }
                else {
                    let mut found: bool = false;
                    let mut found_path = PathBuf::new();
                    if let Ok(path_values) = env::var("PATH") {
                        let paths: Vec<&str> = path_values.split(':').collect();
                        for p in paths {
                            let full_path = Path::new(p).join(&cmd);
                                if full_path.exists() && full_path.is_executable() {
                                    found = true;
                                    found_path = full_path;
                                    break
                        }
                    }
                }
                    if found {
                        println!("{} is {}", cmd, found_path.display());
                    } else {
                        println!("{}: not found", cmd);
                    }
                }
            }
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match Command::parse(&input) {
            Ok(cmd) => cmd.execute(),
            Err(e) => {
                if !e.is_empty() {
                    eprintln!("{}", e);
                }
            }
        }


    }
}
