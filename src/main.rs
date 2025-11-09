mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;

use std::{env, fs, path::Path};

use error::LangError;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), LangError> {
    let path = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            return Err(LangError::Runtime(
                "Usage: fippli_lang <source-file>".to_string(),
                None,
            ))
        }
    };

    let source_path = Path::new(&path);
    if !source_path.exists() {
        return Err(LangError::Runtime(
            format!("Source file '{}' not found", path),
            None,
        ));
    }

    let source = fs::read_to_string(source_path)?;
    let tokens =
        Lexer::with_source_and_file(&source, source.clone(), source_path.to_path_buf()).lex()?;
    let mut parser =
        Parser::with_source_and_file(tokens, source.clone(), source_path.to_path_buf());
    let program = parser.parse_program()?;

    // Set entry point directory for module resolution
    let entry_point_dir = source_path
        .parent()
        .ok_or_else(|| {
            LangError::Runtime("Cannot determine entry point directory".to_string(), None)
        })?
        .to_path_buf();

    let mut interpreter = Interpreter::with_entry_point_dir(entry_point_dir);
    interpreter.eval_program(&program)?;
    Ok(())
}
