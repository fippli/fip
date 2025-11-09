use crate::error::{byte_offset_to_line, LangError, LangResult, Location};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(i64),
    StringLiteral(String),
    Boolean(bool),
    Null,
    Newline,
    Colon,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Ampersand,
    Pipe,
    Dot,
    Spread,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Exclamation,
    Question,
    Eof,
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current_index: usize,
    next_index: usize,
    peeked: Option<char>,
    source: String,
    file_path: PathBuf,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            current_index: 0,
            next_index: 0,
            peeked: None,
            source: String::new(),
            file_path: PathBuf::from("<unknown>"),
        }
    }

    pub fn with_source_and_file(input: &'a str, source: String, file_path: PathBuf) -> Self {
        Self {
            chars: input.chars(),
            current_index: 0,
            next_index: 0,
            peeked: None,
            source,
            file_path,
        }
    }

    fn error_with_location(&self, msg: String, byte_offset: usize) -> LangError {
        let line = byte_offset_to_line(&self.source, byte_offset);
        let location = Some(Location::new(self.file_path.clone(), line));
        LangError::Lexer(msg, location)
    }

    pub fn lex(mut self) -> LangResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                let start = self.current_index;
                self.advance_char();
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    span: start..self.current_index,
                });
                continue;
            }

            if ch.is_whitespace() {
                self.consume_whitespace();
                continue;
            }

            let start = self.current_index;
            let token = match ch {
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(start)?,
                '0'..='9' => self.read_number(start)?,
                '"' => self.read_string(start)?,
                ':' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Colon,
                        span: start..self.current_index,
                    }
                }
                ',' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Comma,
                        span: start..self.current_index,
                    }
                }
                '(' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::LParen,
                        span: start..self.current_index,
                    }
                }
                ')' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::RParen,
                        span: start..self.current_index,
                    }
                }
                '[' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::LBracket,
                        span: start..self.current_index,
                    }
                }
                ']' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::RBracket,
                        span: start..self.current_index,
                    }
                }
                '{' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::LBrace,
                        span: start..self.current_index,
                    }
                }
                '}' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::RBrace,
                        span: start..self.current_index,
                    }
                }
                '.' => {
                    // Check for spread operator (...)
                    self.advance_char(); // Consume the first dot
                    if matches!(self.peek_char(), Some('.')) {
                        self.advance_char(); // Consume the second dot
                        if matches!(self.peek_char(), Some('.')) {
                            self.advance_char(); // Consume the third dot
                            Token {
                                kind: TokenKind::Spread,
                                span: start..self.current_index,
                            }
                        } else {
                            // Two dots but not three - error
                            return Err(self.error_with_location(
                                format!("Unexpected '..' at {}", start),
                                start,
                            ));
                        }
                    } else {
                        // Just a single dot - property access
                        Token {
                            kind: TokenKind::Dot,
                            span: start..self.current_index,
                        }
                    }
                }
                '+' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Plus,
                        span: start..self.current_index,
                    }
                }
                '-' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Minus,
                        span: start..self.current_index,
                    }
                }
                '*' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Star,
                        span: start..self.current_index,
                    }
                }
                '/' => {
                    self.advance_char();
                    if matches!(self.peek_char(), Some('/')) {
                        self.advance_char();
                        self.consume_comment();
                        continue;
                    }
                    Token {
                        kind: TokenKind::Slash,
                        span: start..self.current_index,
                    }
                }
                '&' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Ampersand,
                        span: start..self.current_index,
                    }
                }
                '|' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Pipe,
                        span: start..self.current_index,
                    }
                }
                '=' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Equal,
                        span: start..self.current_index,
                    }
                }
                '<' => {
                    self.advance_char();
                    if matches!(self.peek_char(), Some('=')) {
                        self.advance_char();
                        Token {
                            kind: TokenKind::LessThanEq,
                            span: start..self.current_index,
                        }
                    } else {
                        Token {
                            kind: TokenKind::LessThan,
                            span: start..self.current_index,
                        }
                    }
                }
                '>' => {
                    self.advance_char();
                    if matches!(self.peek_char(), Some('=')) {
                        self.advance_char();
                        Token {
                            kind: TokenKind::GreaterThanEq,
                            span: start..self.current_index,
                        }
                    } else {
                        Token {
                            kind: TokenKind::GreaterThan,
                            span: start..self.current_index,
                        }
                    }
                }
                '!' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Exclamation,
                        span: start..self.current_index,
                    }
                }
                '?' => {
                    self.advance_char();
                    Token {
                        kind: TokenKind::Question,
                        span: start..self.current_index,
                    }
                }
                '\u{2260}' => {
                    // Unicode not equal sign (≠)
                    self.advance_char();
                    Token {
                        kind: TokenKind::NotEqual,
                        span: start..self.current_index,
                    }
                }
                _ => {
                    return Err(self.error_with_location(
                        format!("Unexpected character '{}' at {}", ch, start),
                        start,
                    ))
                }
            };

            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.current_index..self.current_index,
        });

        Ok(tokens)
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    fn consume_comment(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.advance_char();
        }
    }

    fn read_identifier(&mut self, start: usize) -> LangResult<Token> {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ident.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }

        // Consume ! or ? if they follow the identifier (for function names like log!)
        if let Some(ch) = self.peek_char() {
            if ch == '!' || ch == '?' {
                self.advance_char();
                ident.push(ch);
            }
        }

        if ident == "true" {
            return Ok(Token {
                kind: TokenKind::Boolean(true),
                span: start..self.current_index,
            });
        } else if ident == "false" {
            return Ok(Token {
                kind: TokenKind::Boolean(false),
                span: start..self.current_index,
            });
        } else if ident == "null" {
            return Ok(Token {
                kind: TokenKind::Null,
                span: start..self.current_index,
            });
        }

        Ok(Token {
            kind: TokenKind::Identifier(ident),
            span: start..self.current_index,
        })
    }

    fn read_number(&mut self, start: usize) -> LangResult<Token> {
        let mut number = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }

        let value = number.parse::<i64>().map_err(|err| {
            self.error_with_location(
                format!("Invalid number literal '{}': {}", number, err),
                start,
            )
        })?;

        Ok(Token {
            kind: TokenKind::Number(value),
            span: start..self.current_index,
        })
    }

    fn read_string(&mut self, start: usize) -> LangResult<Token> {
        self.advance_char(); // consume opening quote
        let mut content = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.advance_char();
                    return Ok(Token {
                        kind: TokenKind::StringLiteral(content),
                        span: start..self.current_index,
                    });
                }
                '\\' => {
                    self.advance_char();
                    let escaped = match self.peek_char() {
                        Some('"') => '"',
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('\\') => '\\',
                        Some('r') => '\r',
                        Some(other) => {
                            return Err(self.error_with_location(
                                format!("Unsupported escape sequence '\\{}'", other),
                                self.current_index,
                            ))
                        }
                        None => {
                            return Err(self.error_with_location(
                                "Unterminated escape sequence in string".to_string(),
                                self.current_index,
                            ))
                        }
                    };
                    content.push(escaped);
                    self.advance_char();
                }
                _ => {
                    content.push(ch);
                    self.advance_char();
                }
            }
        }

        Err(self.error_with_location("Unterminated string literal".to_string(), start))
    }

    fn peek_char(&mut self) -> Option<char> {
        if let Some(ch) = self.peeked {
            Some(ch)
        } else {
            self.peeked = self.chars.next();
            if let Some(ch) = self.peeked {
                self.next_index = self.current_index + ch.len_utf8();
            }
            self.peeked
        }
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char();
        if let Some(actual) = ch {
            self.current_index = self.next_index;
            self.peeked = None;
            Some(actual)
        } else {
            None
        }
    }
}
