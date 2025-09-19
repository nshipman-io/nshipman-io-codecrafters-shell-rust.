use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use is_executable::IsExecutable;

enum Commands {
    Exit(Option<i32>),
    Echo(Vec<String>),
    Type(String),
    External(String, Vec<String>),
}

struct Builtin {
    name: &'static str,
}

impl Commands {
    const BUILTINS: &'static [Builtin] = &[
        Builtin { name: "exit", },
        Builtin { name: "echo", },
        Builtin { name: "type", }
    ];

    fn is_builtin(name: &str) -> bool {
        Self::BUILTINS.iter().any(|b| b.name == name)
    }

    fn parse(input: &str) -> Result<Commands, String> {
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
                Ok(Commands::Exit(code))
            },
            "echo" => {
                let args = args.iter().map(|s| s.to_string())
                    .collect();
                Ok(Commands::Echo(args))
            },
            "type" => {
                match args.get(0) {
                    Some(cmd) => Ok(Commands::Type(cmd.to_string())),
                    None => Err("type: missing argument".to_string())
                }
            }
            _ => {
                let args = args.iter().map(|s| s.to_string()).collect();
                Ok(Commands::External(cmd.to_string(), args))
            }
        }
    }

    fn execute(self) {
        match self {
            Commands::Exit(code) => {
                std::process::exit(code.unwrap_or(0));
            }
            Commands::Echo(args) => {
                println!("{}", args.join(" "));
            }

            Commands::Type(cmd) => {
                if Self::is_builtin(&cmd) {
                    println!("{} is a shell builtin", cmd);
                } else if let Some(path) = self::Commands::find_cmd(&cmd) {
                    println!("{} is {}", cmd, path.display());
                } else {
                    println!("{}: not found", cmd);
                }
            }

            Commands::External(cmd, args) => {
                if let Some(path) = Commands::find_cmd(&cmd) {
                   match Command::new(&cmd)
                       .args(&args)
                       .env("PATH", env::var("PATH").unwrap_or_default())
                       .status()
                   {
                       Ok(status) => {
                           if !status.success() {
                               std::process::exit(status.code().unwrap_or(1));
                           }
                       }
                       Err(_) => {
                           match Command::new(&path)
                               .args(&args)
                               .status()
                           {
                               Ok(status) => {
                                   if !status.success() {
                                       std::process::exit(status.code().unwrap_or(1));
                                   }
                               }
                               Err(e) => {
                                    eprintln! ("{}: {}", cmd, e);
                                }
                           }
                       }
                   }
                } else {
                    println!("{}: not found", cmd);
                }
            }
        }
    }

    fn find_cmd(cmd: &str) -> Option<PathBuf> {
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
            Some(found_path)
        } else {
            None
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match Commands::parse(&input) {
            Ok(cmd) => cmd.execute(),
            Err(e) => {
                if !e.is_empty() {
                    eprintln!("{}", e);
                }
            }
        }


    }
}
