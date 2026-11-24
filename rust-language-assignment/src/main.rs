use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, Editor};

mod completion;
mod expand;
use completion::ShellHelper;
use expand::expand;

fn main()  {
    let mut cwd: PathBuf = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("shell: cannot read current directory: {e}");
            return;
        }
    };

    // List: first Tab completes the common prefix, second Tab lists the options
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl: Editor<ShellHelper, DefaultHistory> = match Editor::with_config(config) {
        Ok(rl) => rl,
        Err(e) => {
            println!("shell: cannot start line editor: {e}");
            return;
        }
    };
    rl.set_helper(Some(ShellHelper { cwd: cwd.clone() }));

    // history is kept in ~/.rust_shell_history between sessions
    let history_file = match env::var("HOME") {
        Ok(home) => Some(PathBuf::from(home).join(".rust_shell_history")),
        Err(_) => None,
    };
    if let Some(file) = &history_file {
        // the file doesn't exist on the first run, so a failed load is fine
        let _ = rl.load_history(file);
    }

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

        match cmd {
            "exit" => break,
            "cd" => {
                cd(&mut cwd, &args);
                // completion has to know the new directory too
                if let Some(helper) = rl.helper_mut() {
                    helper.cwd = cwd.clone();
                }
            }
            "ls" => ls(&cwd, &args),
            _ => run_external(&cwd, cmd, &args),
        }

    }

    if let Some(file) = &history_file {
        if let Err(e) = rl.save_history(file) {
            println!("shell: cannot save history: {e}");
        }
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
    if args.is_empty() {
        list_dir(cwd);
        return;
    }
    for arg in args {
        let path = cwd.join(arg);
        if path.is_file() {
            println!("{arg}");
        } else {
            if args.len() > 1 {
                println!("{arg}:");
            }
            list_dir(&path);
        }
    }
}

fn list_dir(path: &PathBuf) {
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => println!("{}", entry.file_name().to_string_lossy()),
                    Err(e) => println!("ls: {}: {e}", path.display()),
                }
            }
        }
        Err(e) => println!("ls: {}: {e}", path.display()),
    }
}