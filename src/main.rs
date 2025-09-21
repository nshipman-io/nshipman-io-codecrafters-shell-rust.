use std::io::{self, ErrorKind, Write};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use is_executable::IsExecutable;

enum Commands {
    Exit(Option<i32>),
    Echo(Vec<String>),
    Type(String),
    External(String, Vec<String>),
    Pwd(),
    Cd(String),
}

enum SpecialChar {
    DoubleQuote, // "
    SingleQuote, // '
}

#[derive(PartialEq)]
enum QuoteState {
    None,
    InSingleQuote,
    InDoubleQuote,
}

impl SpecialChar {
    fn from_char(ch: char) ->Option<SpecialChar> {
        match ch {
            '\'' => Some(SpecialChar::SingleQuote),
            '"' => Some(SpecialChar::DoubleQuote),
            _ => None
        }
    }
}

struct Builtin {
    name: &'static str,
}

impl Commands {
    const BUILTINS: &'static [Builtin] = &[
        Builtin { name: "exit", },
        Builtin { name: "echo", },
        Builtin { name: "type", },
        Builtin { name: "pwd", },
        Builtin { name: "cd", },
    ];

    fn is_builtin(name: &str) -> bool {
        Self::BUILTINS.iter().any(|b| b.name == name)
    }

    fn parse(input: &str) -> Result<Commands, String> {
        //let parts: Vec<&str> = input.trim().split_whitespace().collect();
        //let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let parts: Vec<String> = tokenize(input);
        if parts.is_empty() {
            return Err("".to_string());
        }
        // TODO: Construct the

        let cmd = &parts[0];
        let args = &parts[1..];
        match cmd.as_str() {
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
            },
            "cd" => {
                match args.get(0) {
                    Some(path) => Ok(Commands::Cd(path.to_string())),
                    None => {
                        let path = "~";
                        Ok(Commands::Cd(path.to_string()))
                        }
                    }
                },
            "pwd" => Ok(Commands::Pwd()),
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
                                    eprintln!("{}: {}", cmd, e);
                                }
                            }
                        }
                    }
                } else {
                    println!("{}: not found", cmd);
                }
            }
            Commands::Pwd() => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => {
                        eprintln!("pwd: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            Commands::Cd(path) => {
                let target = if path.starts_with("~/"){
                   match env::var("HOME") {
                       Ok(home) => path.replace("~", &home),
                       Err(e) => {
                           eprintln!("cd: HOME: {}", e);
                           return;
                       }
                   }
                } else if path == "~" {
                    match env::var("HOME") {
                        Ok(home) => home,
                        Err(e) => {
                            eprintln!("cd: HOME: {}",e );
                            return;
                        }
                    }
                } else {
                    path.clone()
                };

                if let Err(e) = env::set_current_dir(&target) {
                    match e.kind() {
                        ErrorKind::NotFound => eprintln!("cd: {}: No such file or directory", target),
                        _ => {
                            eprintln!("cd: {}: {}", target, e);
                        }
                    }
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

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut state = QuoteState::None;

    for ch in input.chars() {
        // Handle state when a quote is left open
        if ch == '\'' && state == QuoteState::None {
            state = QuoteState::InSingleQuote;
            continue;
        } else if ch == '\'' && state == QuoteState::InSingleQuote {
            state = QuoteState::None;
            continue;
        }

        if ch == '\"' && state ==QuoteState::None {
            state = QuoteState::InDoubleQuote;
            continue;
        } else if ch == '\"' && state == QuoteState::InDoubleQuote {
            state = QuoteState::None;
            continue;
        } else if ch == '\'' && state == QuoteState::InDoubleQuote {
            current_token.push(ch);
            continue;
        }

        if ch == ' ' && state == QuoteState::None {
            if !current_token.is_empty() {
                tokens.push(current_token.clone().to_string());
                current_token.clear();
            }
            continue;
        }
        current_token.push(ch);

    }
    if !current_token.is_empty() {
        tokens.push(current_token.trim().to_string());
    }
    tokens
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
