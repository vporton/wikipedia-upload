use std::{fs, process};
use std::fmt::{Display, Formatter};
use std::path::Path;
use filetime::set_file_mtime;
use log::{debug, error};
use walkdir::WalkDir;
use clap::Parser;
use filetime::FileTime;

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

#[derive(Parser, Debug)]
struct Args {
    /// input directory
    source_file: String,

    /// output directory
    destination_directory: String,
}

fn main() {
    env_logger::builder().init();
    if let Err(err) = almost_main() {
        error!("{err}");
        process::exit(1);
    }
}

fn almost_main() -> Result<(), MyError> {
    let args = Args::parse();

    let mtime = fs::metadata(args.source_file.clone())?;

    for entry in WalkDir::new(Path::new(&args.destination_directory))
        // .sort_by_file_name() // keep the order deterministic, because we overwrite files
        .into_iter()
    {
        let entry = entry?;
        debug!("Setting file {} mtime", entry.path().to_str().unwrap());
        set_file_mtime(entry.path(), FileTime::from_last_modification_time(&mtime))?;
    }

    Ok(())
}
