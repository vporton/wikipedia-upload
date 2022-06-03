use std::{env, process};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use lazy_static::lazy_static;
use walkdir::WalkDir;
use maplit::hashset;
use regex::Regex;

#[derive(Debug)]
enum MyError {
    IO(std::io::Error),
    WalkDir(walkdir::Error),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "I/O: {err}"),
            Self::WalkDir(err) => write!(f, "Walking dir: {err}"),
        }
    }
}

impl From<walkdir::Error> for MyError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkDir(value)
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

fn main() {
    if let Err(err) = almost_main() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn almost_main() -> Result<(), MyError> {
    if env::args().len() != 2 {
        eprintln!("Usage: indexer <DIR>");
        process::exit(1);
    }
    for entry in WalkDir::new(Path::new(&env::args().nth(1).unwrap()))
        .sort_by_file_name() // keep the order deterministic, because we overwrite files
        .into_iter()
        .filter_entry(|entry| !entry.path_is_symlink())
    {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            println!("{}", entry.path().to_str().unwrap());
            index_file(&entry.path())?;
        }
    }

    Ok(())
}

lazy_static! {
    static ref WIKIPEDIA_REMOVE: Regex = Regex::new(r"(?s)<!--htdig_noindex-->.*?<!--/htdig_noindex-->").unwrap();
}

#[cfg(test)]
mod tests {
    use crate::WIKIPEDIA_REMOVE;

    #[test]
    fn test_regex() {
        assert_eq!(WIKIPEDIA_REMOVE.replace("u\nx<!--htdig_noindex-->a\nb<!--/htdig_noindex-->y", ""), "u\nxy");
    }
}

fn index_file(path: &Path) -> Result<(), MyError> {
    let mut input = File::open(path.clone())?; // uncompressed text file
    let mut cleaned= Vec::new();
    input.read_to_end(&mut cleaned)?;
    let cleaned = String::from_utf8_lossy(&*cleaned.as_slice());
    let cleaned = WIKIPEDIA_REMOVE.replace(&*cleaned, "");
    let cleaned = ammonia::Builder::default()
        .tags(hashset![])
        .clean_content_tags(hashset!["head", "script"])
        .clean(&*cleaned)
        .to_string();
    println!("{}", cleaned);
    Ok(())
}