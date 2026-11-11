use std::path::PathBuf;
use std::{env, fs};

pub fn expand(cwd: &PathBuf, args: &[&str]) -> Vec<String> {
    let mut result = Vec::new();
    for arg in args {
        if *arg == "." {
            result.push(cwd.display().to_string());
        } else if *arg == "~" || arg.starts_with("~/") {
            let home = env::var("HOME").unwrap_or_default();
            result.push(arg.replacen('~', &home, 1));
        } else if arg.contains('*') {
            let found = glob(cwd, arg);
            if found.is_empty() {
                result.push(arg.to_string());
            } else {
                result.extend(found);
            }
        } else {
            result.push(arg.to_string());
        }
    }
    return result
}

fn glob(cwd: &PathBuf, pattern: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(cwd) {
        for entry in entries {
            let name = entry.unwrap().file_name().to_string_lossy().to_string();
            // check for hidden files only
            if name.starts_with('.') && !pattern.starts_with('.') {
                continue;
            }
            if matches(pattern, &name) {
                names.push(name);
            }
        }
    }
    names.sort();
    return names;
}

fn matches(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !name.starts_with(first) {
        return  false;
    }
    let mut rest = &name[first.len()..];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}
