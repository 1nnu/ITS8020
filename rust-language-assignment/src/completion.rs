use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, fs};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

const BUILTINS: [&str; 3] = ["cd", "exit", "ls"];

pub struct ShellHelper {
    pub cwd: PathBuf,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let before = &line[..pos];
        // the word being completed starts after the last space
        let start = match before.rfind(' ') {
            Some(i) => i + 1,
            None => 0,
        };
        let word = &before[start..];
        let is_command = before[..start].trim().is_empty();

        let candidates = if is_command && !word.contains('/') {
            complete_command(word)
        } else {
            complete_path(&self.cwd, word)
        };
        Ok((start, candidates))
    }
}

// rustyline needs these too; the defaults do nothing for now
impl Hinter for ShellHelper {
    type Hint = String;
}
impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Helper for ShellHelper {}

fn complete_command(prefix: &str) -> Vec<Pair> {
    let mut names: Vec<String> = Vec::new();
    for builtin in BUILTINS {
        if builtin.starts_with(prefix) {
            names.push(builtin.to_string());
        }
    }

    // every executable in every PATH directory is a possible command
    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':') {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => continue,
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(prefix) && is_executable(&entry.path()) {
                    names.push(name);
                }
            }
        }
    }

    names.sort();
    names.dedup();
    names
        .into_iter()
        .map(|name| Pair {
            display: name.clone(),
            replacement: format!("{name} "),
        })
        .collect()
}

fn is_executable(path: &PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

fn complete_path(cwd: &PathBuf, word: &str) -> Vec<Pair> {
    // split "src/ma" into the directory part "src/" and the prefix "ma"
    let (dir_part, prefix) = match word.rfind('/') {
        Some(i) => (&word[..i + 1], &word[i + 1..]),
        None => ("", word),
    };
    let dir = if dir_part.starts_with("~/") {
        match env::var("HOME") {
            Ok(home) => PathBuf::from(home).join(&dir_part[2..]),
            Err(_) => return Vec::new(),
        }
    } else {
        cwd.join(dir_part)
    };

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(prefix) {
            continue;
        }
        // hidden files only complete when the user typed the `.`
        if name.starts_with('.') && !prefix.starts_with('.') {
            continue;
        }
        // directories end in `/` so you can keep tabbing into them
        if entry.path().is_dir() {
            candidates.push(Pair {
                display: format!("{name}/"),
                replacement: format!("{dir_part}{name}/"),
            });
        } else {
            candidates.push(Pair {
                display: name.clone(),
                replacement: format!("{dir_part}{name} "),
            });
        }
    }
    candidates.sort_by(|a, b| a.display.cmp(&b.display));
    candidates
}
