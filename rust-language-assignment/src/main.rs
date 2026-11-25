use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;

mod expand;
mod helper;
use expand::expand;
use helper::ShellHelper;

fn main()  {
    let mut cwd: PathBuf = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("shell: cannot read current directory: {e}");
            return;
        }
    };

    let mut rl: Editor<ShellHelper, DefaultHistory> = match Editor::new() {
        Ok(rl) => rl,
        Err(e) => {
            println!("shell: cannot start line editor: {e}");
            return;
        }
    };
    rl.set_helper(Some(ShellHelper));

    // history is kept in ~/.rust_shell_history between sessions
    let history_file = match env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".rust_shell_history"),
        Err(_) => PathBuf::from(".rust_shell_history"),
    };
    // the file doesn't exist on the first run, so a failed load is fine
    let _ = rl.load_history(&history_file);

    loop {
        let prompt = format!("{}> ", cwd.display());
        let line = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue, // Ctrl-C clears the line
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(e) => {
                println!("shell: cannot read input: {e}");
                break;
            }
        };

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(input);

        let mut parts = input.split_whitespace();
        let cmd = match parts.next() {
            Some(cmd) => cmd,
            None => continue,
        };
        let args: Vec<&str> = parts.collect();

        let expanded = expand(&cwd, &args);
        let args: Vec<&str> = expanded.iter().map(|s| s.as_str()).collect();
        let (args, stdout, stderr) = parse_redirects(&args);

        match cmd {
            "exit" => break,
            "cd" => cd(&mut cwd, &args),
            "ls" => ls(&cwd, &args, stdout),
            _ => run_external(&cwd, cmd, &args, stdout, stderr),
        }
    }

    if let Err(e) = rl.save_history(&history_file) {
        println!("shell: cannot save history: {e}");
    }
}

// takes `> file` and `2> file` out of the args and returns the file names
fn parse_redirects<'a>(args: &[&'a str]) -> (Vec<&'a str>, Option<&'a str>, Option<&'a str>) {
    let mut rest = Vec::new();
    let mut stdout = None;
    let mut stderr = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            ">" => {
                stdout = args.get(i + 1).copied();
                i += 2;
            }
            "2>" => {
                stderr = args.get(i + 1).copied();
                i += 2;
            }
            arg => {
                rest.push(arg);
                i += 1;
            }
        }
    }
    (rest, stdout, stderr)
}

fn run_external(cwd: &PathBuf, cmd: &str, args: &[&str], stdout: Option<&str>, stderr: Option<&str>) {
    let mut command = Command::new(cmd);
    command.args(args).current_dir(cwd);

    if let Some(name) = stdout {
        match File::create(cwd.join(name)) {
            Ok(file) => {
                command.stdout(file);
            }
            Err(e) => {
                println!("{name}: {e}");
                return;
            }
        }
    }
    if let Some(name) = stderr {
        match File::create(cwd.join(name)) {
            Ok(file) => {
                command.stderr(file);
            }
            Err(e) => {
                println!("{name}: {e}");
                return;
            }
        }
    }

    match command.spawn() {
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

fn ls(cwd: &PathBuf, args: &[&str], stdout: Option<&str>) {
    // write to the file after `>` if there is one, otherwise to the terminal
    let mut out: Box<dyn Write> = match stdout {
        Some(name) => match File::create(cwd.join(name)) {
            Ok(file) => Box::new(file),
            Err(e) => {
                println!("{name}: {e}");
                return;
            }
        },
        None => Box::new(io::stdout()),
    };

    if args.is_empty() {
        list_dir(cwd, &mut out);
        return;
    }
    for arg in args {
        let path = cwd.join(arg);
        if path.is_file() {
            let _ = writeln!(out, "{arg}");
        } else {
            if args.len() > 1 {
                let _ = writeln!(out, "{arg}:");
            }
            list_dir(&path, &mut out);
        }
    }
}

fn list_dir(path: &PathBuf, out: &mut Box<dyn Write>) {
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let _ = writeln!(out, "{}", entry.file_name().to_string_lossy());
                    }
                    Err(e) => println!("ls: {}: {e}", path.display()),
                }
            }
        }
        Err(e) => println!("ls: {}: {e}", path.display()),
    }
}
