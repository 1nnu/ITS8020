use std::borrow::Cow;
use std::path::Path;
use std::{env, fs};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

const BUILTINS: [&str; 3] = ["cd", "exit", "ls"];
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

pub struct ShellHelper;

impl Completer for ShellHelper {
    type Candidate = Pair;

    // completes the command name (the first word) on Tab
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let word = &line[..pos];
        if word.contains(' ') {
            return Ok((0, Vec::new()));
        }

        let mut candidates = Vec::new();
        for name in all_commands() {
            if name.starts_with(word) {
                candidates.push(Pair {
                    display: name.clone(),
                    replacement: name,
                });
            }
        }
        Ok((0, candidates))
    }
}

impl Highlighter for ShellHelper {
    // colors the command: green if it exists, red if it doesn't
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let end = line.find(' ').unwrap_or(line.len());
        let (cmd, rest) = line.split_at(end);
        let color = if is_command(cmd) { GREEN } else { RED };
        Cow::Owned(format!("{color}{cmd}{RESET}{rest}"))
    }

    // redraw the colors after every key press
    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}


impl Hinter for ShellHelper {
    type Hint = String;
}
impl Validator for ShellHelper {}
impl Helper for ShellHelper {}

// builtins plus every program in the PATH directories
fn all_commands() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for builtin in BUILTINS {
        names.push(builtin.to_string());
    }
    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':') {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    names.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn is_command(cmd: &str) -> bool {
    if BUILTINS.contains(&cmd) {
        return true;
    }
    match env::var("PATH") {
        Ok(path) => path.split(':').any(|dir| Path::new(dir).join(cmd).is_file()),
        Err(_) => false,
    }
}
