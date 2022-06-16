use std::process;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir, File};
use std::io::{Read, Write};
use std::path::{Path, StripPrefixError};
use lazy_static::lazy_static;
use walkdir::WalkDir;
use maplit::hashset;
use regex::{Regex, Split};
use clap::Parser;
use log::{debug, error};

#[derive(Debug)]
struct MyUnicodeError;

impl MyUnicodeError {
    pub fn new() -> Self {
        Self { }
    }
}

impl Display for MyUnicodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unicode error")
    }
}

#[derive(Debug)]
enum MyError {
    IO(std::io::Error),
    WalkDir(walkdir::Error),
    StripPrefix(StripPrefixError),
    MyUnicode(MyUnicodeError),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "I/O: {err}"),
            Self::WalkDir(err) => write!(f, "Walking dir: {err}"),
            Self::StripPrefix(err) => write!(f, "Strip file prefix: {err}"),
            Self::MyUnicode(err) => write!(f, "Unicode error: {err}"),
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

impl From<StripPrefixError> for MyError {
    fn from(value: StripPrefixError) -> Self {
        Self::StripPrefix(value)
    }
}

impl From<MyUnicodeError> for MyError {
    fn from(value: MyUnicodeError) -> Self {
        Self::MyUnicode(value)
    }
}

fn main() {
    env_logger::builder().init();
    if let Err(err) = almost_main() {
        error!("{err}");
        process::exit(1);
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// input directory
    input_dir: String,

    /// output directory
    output_dir: String,

    /// maximum length of search listing
    #[clap(short, long, default_value = "500")]
    max_entries: u32,
}

fn almost_main() -> Result<(), MyError> {
    let args = Args::parse();

    let _error = create_dir(args.output_dir.clone());

    for entry in WalkDir::new(Path::new(&args.input_dir))
        .sort_by_file_name() // keep the order deterministic, because we overwrite files
        .into_iter()
        .filter_entry(|entry| !entry.path_is_symlink())
    {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            index_file(&entry.path(), &args)?;
        }
    }

    Ok(())
}

lazy_static! {
    static ref NBSP: Regex = Regex::new(r"&nbsp;").unwrap();
    static ref CELL_END: Regex = Regex::new(r"</th>|</td>").unwrap();
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

fn index_file(path: &Path, args: &Args) -> Result<(), MyError> {
    debug!("Indexing file {}", path.to_str().unwrap());
    let mut input = File::open(path.clone())?; // uncompressed text file
    let mut cleaned= Vec::new();
    input.read_to_end(&mut cleaned)?;
    let cleaned = String::from_utf8_lossy(&*cleaned.as_slice());
    let cleaned = NBSP.replace_all(&*cleaned, " ");
    let cleaned = CELL_END.replace_all(&*cleaned, " ");
    let cleaned = WIKIPEDIA_REMOVE.replace_all(&*cleaned, "");
    let cleaned = ammonia::Builder::default()
        .tags(hashset![])
        .clean_content_tags(hashset!["head", "script"])
        .clean(&*cleaned)
        .to_string();
    let mut word_counts = HashMap::new();
    for word in tokenize(cleaned.as_str()) {
        if word.is_empty() {
            continue;
        }
        let word = word.to_lowercase();
        word_counts.entry(word.clone()).and_modify(|e| *e += 1).or_insert(1);
        if word_counts[word.as_str()] <= args.max_entries as u64 {
            let rel_path = path.strip_prefix(Path::new(args.input_dir.as_str()))?;
            let output_path = Path::new(args.output_dir.as_str()).join(Path::new(word.as_str()));
            // Ignore "File name too long" error:
            match File::options().create(true).append(true).open(output_path) {
                Ok(mut file) => {
                    let s = rel_path.to_str().ok_or(MyUnicodeError::new())?.to_string() + "\0";
                    file.write(s.as_bytes())?;
                }
                Err(err) => {
                    if err.raw_os_error().unwrap() != 36 { // Filename too long // TODO: Non-Linux systems
                        Err::<_, MyError>(err.into())?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn tokenize(s: &str) -> Split {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\W").unwrap();
    }
    RE.split(s)
}