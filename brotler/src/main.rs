use std::{env, process};
use std::fmt::{Display, Formatter};
use std::fs::{File, rename};
use std::io::copy;
use std::path::Path;
use brotlic::{BrotliEncoderOptions, CompressorWriter, Quality, SetParameterError, WindowSize};
use tempfile::{NamedTempFile, TempPath};
use walkdir::WalkDir;

#[derive(Debug)]
enum MyError {
    IO(std::io::Error),
    WalkDir(walkdir::Error),
    SetParameter(SetParameterError),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "I/O: {err}"),
            Self::WalkDir(err) => write!(f, "Walking dir: {err}"),
            Self::SetParameter(err) => write!(f, "Setting parameter: {err}"),
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

impl From<SetParameterError> for MyError {
    fn from(value: SetParameterError) -> Self {
        Self::SetParameter(value)
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
        compress_file(&entry.path())?;
    }

    Ok(())
}

fn compress_file(path: &Path) -> Result<(), MyError> {
    let mut input = File::open(path.clone())?; // uncompressed text file
    let output_file = NamedTempFile::new()?;
    let output_path: TempPath = output_file.into_temp_path();
    let output_path: &Path = output_path.as_ref();
    let output = File::create(output_path)?; // compressed text output file

    let encoder = BrotliEncoderOptions::new()
        .quality(Quality::best())
        .window_size(WindowSize::new(24)?)
        .build()?;

    let mut output_compressed = CompressorWriter::with_encoder(encoder, output);

    copy(&mut input, &mut output_compressed)?;
    rename(output_path, path)?;
    Ok(())
}