use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main()  {
    let mut buffer: String = String::new();
    let mut cwd: PathBuf = env::current_dir().unwrap();
    loop {
        print!("{}> ", cwd.display());
        io::stdout().flush().unwrap();

        buffer.clear();
        let bytes = io::stdin().read_line(&mut buffer).unwrap();

        if bytes == 0 {
            println!();
            break;
        }

        let input = buffer.trim();
        if input.is_empty() {
            continue;
        }

        let mut parts = input.split_whitespace();
        let cmd = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        match cmd {
            "exit" => break,
            "cd" => cd(&mut cwd, &args),
            "ls" => ls(&cwd, &args),
            _ => run_external(&cwd, cmd, &args),
        }

        println!("input: {input}")
    }
}

fn run_external(cwd: &PathBuf, cmd: &str, args: &[&str]) {
    match Command::new(cmd).args(args).current_dir(cwd).spawn() {
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                println!("{cmd}: {e}");
            }
        }
        Err(e) => println!("{cmd}: {e}"),
    }
}

fn cd(cwd: &mut PathBuf, args: &[&str]) {
    if args.len() != 1 {
        println!("cd: expected exactly one argument");
        return;
    }
    let target = args[0];
    match cwd.join(target).canonicalize() {
        Ok(path) if path.is_dir() => *cwd = path,
        Ok(_) => println!("cd: {target}: not a directory"),
        Err(e) => println!("cd: {target}: {e}"),
    }
}

fn ls(cwd: &PathBuf, args: &[&str]) {
    if args.len() > 1 {
        println!("ls: expected at most one argument");
        return;
    }
    let path = if args.is_empty() {
        cwd.clone()
    } else {
        cwd.join(args[0])
    };

    match fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry.unwrap().file_name().to_string_lossy());
            }
        }
        Err(e) => println!("ls: {}: {e}", path.display()),
    }
}