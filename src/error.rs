use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
}

impl Location {
    pub fn new(file: PathBuf, line: usize) -> Self {
        Self { file, line }
    }
}

#[derive(Debug)]
pub enum LangError {
    Io(std::io::Error),
    Lexer(String, Option<Location>),
    Parser(String, Option<Location>),
    Runtime(String, Option<Location>),
}

pub type LangResult<T> = Result<T, LangError>;

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LangError::Io(err) => write!(f, "I/O error: {}", err),
            LangError::Lexer(msg, location) => {
                if let Some(loc) = location {
                    // Extract just the filename from the path
                    let filename = loc
                        .file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| loc.file.to_str().unwrap_or("<unknown>"));
                    write!(
                        f,
                        "Lex error: {}\nFile: {} line {}",
                        msg, filename, loc.line
                    )
                } else {
                    write!(f, "Lex error: {}", msg)
                }
            }
            LangError::Parser(msg, location) => {
                if let Some(loc) = location {
                    // Extract just the filename from the path
                    let filename = loc
                        .file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| loc.file.to_str().unwrap_or("<unknown>"));
                    write!(
                        f,
                        "Parse error: {}\nFile: {} line {}",
                        msg, filename, loc.line
                    )
                } else {
                    write!(f, "Parse error: {}", msg)
                }
            }
            LangError::Runtime(msg, location) => {
                if let Some(loc) = location {
                    // Extract just the filename from the path
                    let filename = loc
                        .file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| loc.file.to_str().unwrap_or("<unknown>"));
                    write!(
                        f,
                        "Runtime error: {}\nFile: {} line {}",
                        msg, filename, loc.line
                    )
                } else {
                    write!(f, "Runtime error: {}", msg)
                }
            }
        }
    }
}

impl std::error::Error for LangError {}

impl From<std::io::Error> for LangError {
    fn from(value: std::io::Error) -> Self {
        LangError::Io(value)
    }
}

pub fn byte_offset_to_line(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .chars()
        .filter(|&c| c == '\n')
        .count()
        + 1
}
