use std::{env, process};
use std::fmt::{Display, Formatter};
use std::path::Path;
use walkdir::{Error, WalkDir};

#[derive(Debug)]
enum MyError {
    WalkDir(walkdir::Error),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WalkDir(err) => write!(f, "Walking dir: {err}"),
        }
    }
}

impl From<walkdir::Error> for MyError {
    fn from(value: Error) -> Self {
        Self::WalkDir(value)
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
        eprintln!("Usage: brotler <DIR>");
        process::exit(1);
    }
    for entry in WalkDir::new("foo")
        .sort_by_file_name() // keep the order deterministic, because we overwrite files
        .into_iter()
        .filter_entry(|e| !e.path_is_symlink())
    {
        let entry = entry?;
        compress_file(&entry.path())
    }

    Ok(())
}

fn compress_file(path: &Path) {

}