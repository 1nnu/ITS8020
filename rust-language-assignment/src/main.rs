use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{self, Command};
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
    let mut aliases: HashMap<String, String> = HashMap::new();

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
        // saved after every line, so `exit` anywhere in a chain keeps the history
        if let Err(e) = rl.save_history(&history_file) {
            println!("shell: cannot save history: {e}");
        }

        // `a ; b` runs both, `a && b` runs b only if a succeeded
        for group in input.split(';') {
            for command in group.split("&&") {
                if !run_command(command, &mut cwd, &mut aliases) {
                    break;
                }
            }
        }
    }
}

// runs one command and returns whether it succeeded
fn run_command(command: &str, cwd: &mut PathBuf, aliases: &mut HashMap<String, String>) -> bool {
    let line = replace_alias(command, aliases);
    let mut parts = line.split_whitespace();
    let cmd = match parts.next() {
        Some(cmd) => cmd,
        None => return true, // empty command, e.g. after a trailing `;`
    };
    let args: Vec<&str> = parts.collect();

    // alias gets its args before expansion, so `alias x=ls *` keeps the `*`
    if cmd == "alias" {
        return alias(aliases, &args);
    }

    let expanded = expand(cwd, &args);
    let args: Vec<&str> = expanded.iter().map(|s| s.as_str()).collect();
    let (args, stdout, stderr) = parse_redirects(&args);

    match cmd {
        "exit" => process::exit(0),
        "cd" => cd(cwd, &args),
        "ls" => ls(cwd, &args, stdout),
        "pwd" => pwd(cwd, stdout),
        _ => run_external(cwd, cmd, &args, stdout, stderr),
    }
}

// if the first word is an alias, swap it for its value: `ll src` -> `ls -a src`
fn replace_alias(command: &str, aliases: &HashMap<String, String>) -> String {
    let command = command.trim();
    let (first, rest) = match command.split_once(' ') {
        Some((first, rest)) => (first, rest),
        None => (command, ""),
    };
    match aliases.get(first) {
        Some(value) => format!("{value} {rest}"),
        None => command.to_string(),
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

// the file after `>` if there is one, otherwise the terminal
fn open_output(cwd: &PathBuf, stdout: Option<&str>) -> Option<Box<dyn Write>> {
    match stdout {
        Some(name) => match File::create(cwd.join(name)) {
            Ok(file) => Some(Box::new(file)),
            Err(e) => {
                println!("{name}: {e}");
                None
            }
        },
        None => Some(Box::new(io::stdout())),
    }
}

fn run_external(cwd: &PathBuf, cmd: &str, args: &[&str], stdout: Option<&str>, stderr: Option<&str>) -> bool {
    let mut command = Command::new(cmd);
    command.args(args).current_dir(cwd);

    if let Some(name) = stdout {
        match File::create(cwd.join(name)) {
            Ok(file) => {
                command.stdout(file);
            }
            Err(e) => {
                println!("{name}: {e}");
                return false;
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
                return false;
            }
        }
    }

    match command.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => status.success(), // exit code 0 means success
            Err(e) => {
                println!("{cmd}: {e}");
                false
            }
        },
        Err(e) => {
            println!("{cmd}: {e}");
            false
        }
    }
}

// `alias` lists the aliases, `alias ll=ls -a` adds one
fn alias(aliases: &mut HashMap<String, String>, args: &[&str]) -> bool {
    if args.is_empty() {
        for (name, value) in aliases.iter() {
            println!("alias {name}='{value}'");
        }
        return true;
    }
    let text = args.join(" ");
    match text.split_once('=') {
        Some((name, value)) => {
            // allow `alias ll='ls -a'`: the quotes are not part of the command
            let value = value.trim_matches('\'').trim_matches('"');
            aliases.insert(name.to_string(), value.to_string());
            true
        }
        None => {
            println!("alias: usage: alias name=command");
            false
        }
    }
}

fn pwd(cwd: &PathBuf, stdout: Option<&str>) -> bool {
    match open_output(cwd, stdout) {
        Some(mut out) => {
            let _ = writeln!(out, "{}", cwd.display());
            true
        }
        None => false,
    }
}

fn cd(cwd: &mut PathBuf, args: &[&str]) -> bool {
    if args.len() != 1 {
        println!("cd: expected exactly one argument");
        return false;
    }
    let target = args[0];
    match cwd.join(target).canonicalize() {
        Ok(path) if path.is_dir() => {
            *cwd = path;
            true
        }
        Ok(_) => {
            println!("cd: {target}: not a directory");
            false
        }
        Err(e) => {
            println!("cd: {target}: {e}");
            false
        }
    }
}

fn ls(cwd: &PathBuf, args: &[&str], stdout: Option<&str>) -> bool {
    let mut out = match open_output(cwd, stdout) {
        Some(out) => out,
        None => return false,
    };

    if args.is_empty() {
        return list_dir(cwd, &mut out);
    }
    let mut ok = true;
    for arg in args {
        let path = cwd.join(arg);
        if path.is_file() {
            let _ = writeln!(out, "{arg}");
        } else {
            if args.len() > 1 {
                let _ = writeln!(out, "{arg}:");
            }
            if !list_dir(&path, &mut out) {
                ok = false;
            }
        }
    }
    ok
}

fn list_dir(path: &PathBuf, out: &mut Box<dyn Write>) -> bool {
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
            true
        }
        Err(e) => {
            println!("ls: {}: {e}", path.display());
            false
        }
    }
}
