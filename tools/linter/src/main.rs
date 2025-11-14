use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use fippli_lang::ast::{
    BinaryOperator, Expression, Function, ObjectField, ObjectPatternField, Pattern, Program,
    Statement, StringSegment,
};
use fippli_lang::error::{byte_offset_to_line, LangError};
use fippli_lang::lexer::Lexer;
use fippli_lang::parser::Parser;

#[derive(Debug, Clone)]
pub struct LintError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct Linter {
    errors: Vec<LintError>,
    defined_names: HashSet<String>,
    used_names: HashSet<String>,
    exported_names: HashSet<String>,
    source: String,
}

impl Linter {
    pub fn new(source: String) -> Self {
        Self {
            errors: Vec::new(),
            defined_names: HashSet::new(),
            used_names: HashSet::new(),
            exported_names: HashSet::new(),
            source,
        }
    }

    fn error_at(&mut self, offset: usize, message: String, severity: Severity) {
        let line = byte_offset_to_line(&self.source, offset);
        let column = self.source[..offset.min(self.source.len())]
            .chars()
            .rev()
            .take_while(|&c| c != '\n')
            .count()
            + 1;
        self.errors.push(LintError {
            line,
            column,
            message,
            severity,
        });
    }

    pub fn lint(&mut self, program: &Program) -> Vec<LintError> {
        self.errors.clear();
        self.defined_names.clear();
        self.used_names.clear();
        self.exported_names.clear();

        // First pass: collect all definitions and exports
        for stmt in &program.statements {
            self.collect_definitions(stmt);
        }

        // Second pass: check rules and collect usage
        for stmt in &program.statements {
            self.check_statement(stmt);
        }

        self.errors.clone()
    }

    fn collect_definitions(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assignment { pattern, .. } => {
                self.collect_pattern_identifiers(pattern);
            }
            Statement::Function(func) => {
                self.defined_names.insert(func.name.clone());
            }
            Statement::Export(export) => {
                self.exported_names.insert(export.name.clone());
            }
            _ => {}
        }
    }

    fn collect_pattern_identifiers(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => {
                self.defined_names.insert(name.clone());
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    self.collect_pattern_identifiers(p);
                }
            }
            Pattern::Object(fields) => {
                for field in fields {
                    match field {
                        ObjectPatternField::Shorthand(name) => {
                            self.defined_names.insert(name.clone());
                        }
                        ObjectPatternField::Field { pattern, .. } => {
                            self.collect_pattern_identifiers(pattern);
                        }
                    }
                }
            }
        }
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Function(func) => {
                self.check_function(func);
            }
            Statement::Assignment { expr, .. } => {
                self.check_expression(expr);
                self.collect_usage(expr);
            }
            Statement::Expression(expr) => {
                self.check_expression(expr);
                self.collect_usage(expr);
            }
            Statement::Use(_) => {}
            Statement::Export(_) => {}
        }
    }

    fn check_function(&mut self, func: &Function) {
        let has_impure_suffix = func.name.ends_with('!');
        let has_boolean_suffix = func.name.ends_with('?');

        // Check if function marked as impure actually calls impure functions
        if func.impure || has_impure_suffix {
            if !Self::find_impure_call(&func.body) {
                // Use offset 0 as fallback since we don't have location info
                self.error_at(
                    0,
                    format!(
                        "Function '{}' is marked impure but performs no impure operations",
                        func.name
                    ),
                    Severity::Error,
                );
            }
        } else {
            // Check if function calls impure functions but isn't marked impure
            if let Some(impure_call) = Self::find_impure_call_name(&func.body) {
                self.error_at(
                    0,
                    format!(
                        "Function '{}' must be declared impure (end the name with '!') to call '{}'",
                        func.name, impure_call
                    ),
                    Severity::Error,
                );
            }
        }

        // Check boolean suffix
        if has_boolean_suffix {
            if !Self::returns_boolean(&func.body) {
                self.error_at(
                    0,
                    format!("Function '{}' must return a boolean value", func.name),
                    Severity::Error,
                );
            }
        }

        // Check expression for other issues
        self.check_expression(&func.body);
        self.collect_usage(&func.body);
    }

    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Lambda { body, impure, .. } => {
                if *impure {
                    if !Self::find_impure_call(body.as_ref()) {
                        self.error_at(
                            0,
                            "Anonymous function is marked impure but performs no impure operations"
                                .to_string(),
                            Severity::Error,
                        );
                    }
                } else {
                    if let Some(impure_call) = Self::find_impure_call_name(body.as_ref()) {
                        self.error_at(
                            0,
                            format!(
                                "Anonymous function must be marked impure (use '!') to call '{}'",
                                impure_call
                            ),
                            Severity::Error,
                        );
                    }
                }
                self.check_expression(body.as_ref());
            }
            Expression::Call { callee, args } => {
                self.check_expression(callee.as_ref());
                for arg in args {
                    self.check_expression(arg);
                }
            }
            Expression::Block(exprs) => {
                for expr in exprs {
                    self.check_expression(expr);
                }
            }
            Expression::Object(fields) => {
                for field in fields {
                    match field {
                        ObjectField::Field { value, .. } => {
                            self.check_expression(value);
                        }
                        ObjectField::Spread(expr) => {
                            self.check_expression(expr);
                        }
                    }
                }
            }
            Expression::Spread(expr) => {
                self.check_expression(expr.as_ref());
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.check_expression(elem);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression(left.as_ref());
                self.check_expression(right.as_ref());
            }
            Expression::PropertyAccess { object, .. } => {
                self.check_expression(object.as_ref());
            }
            Expression::String(template) => {
                for segment in &template.segments {
                    if let StringSegment::Expr(expr) = segment {
                        self.check_expression(expr);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_usage(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.used_names.insert(name.clone());
            }
            Expression::Call { callee, args } => {
                self.collect_usage(callee.as_ref());
                for arg in args {
                    self.collect_usage(arg);
                }
            }
            Expression::Block(exprs) => {
                for expr in exprs {
                    self.collect_usage(expr);
                }
            }
            Expression::Lambda { body, .. } => {
                self.collect_usage(body.as_ref());
            }
            Expression::Object(fields) => {
                for field in fields {
                    match field {
                        ObjectField::Field { value, .. } => {
                            self.collect_usage(value);
                        }
                        ObjectField::Spread(expr) => {
                            self.collect_usage(expr);
                        }
                    }
                }
            }
            Expression::Spread(expr) => {
                self.collect_usage(expr.as_ref());
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.collect_usage(elem);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_usage(left.as_ref());
                self.collect_usage(right.as_ref());
            }
            Expression::PropertyAccess { object, .. } => {
                self.collect_usage(object.as_ref());
            }
            Expression::String(template) => {
                for segment in &template.segments {
                    if let StringSegment::Expr(expr) = segment {
                        self.collect_usage(expr);
                    }
                }
            }
            _ => {}
        }
    }

    fn find_impure_call(expr: &Expression) -> bool {
        match expr {
            Expression::Call { callee, args } => {
                if let Some(name) = Self::identifier_name(callee.as_ref()) {
                    if name.ends_with('!') {
                        return true;
                    }
                }
                Self::find_impure_call(callee.as_ref())
                    || args.iter().any(|arg| Self::find_impure_call(arg))
            }
            Expression::Identifier(name) => name.ends_with('!'),
            Expression::Block(exprs) => exprs.iter().any(|e| Self::find_impure_call(e)),
            Expression::Lambda { body, .. } => Self::find_impure_call(body.as_ref()),
            Expression::Object(fields) => fields.iter().any(|f| match f {
                ObjectField::Field { value, .. } => Self::find_impure_call(value),
                ObjectField::Spread(expr) => Self::find_impure_call(expr),
            }),
            Expression::Spread(expr) => Self::find_impure_call(expr.as_ref()),
            Expression::List(elements) => elements.iter().any(|e| Self::find_impure_call(e)),
            Expression::Binary { left, right, .. } => {
                Self::find_impure_call(left.as_ref()) || Self::find_impure_call(right.as_ref())
            }
            Expression::PropertyAccess { object, property } => {
                // Check if property name ends with '!' (impure method call)
                if property.ends_with('!') {
                    true
                } else {
                    Self::find_impure_call(object.as_ref())
                }
            }
            Expression::String(template) => template
                .segments
                .iter()
                .any(|s| matches!(s, StringSegment::Expr(e) if Self::find_impure_call(e))),
            _ => false,
        }
    }

    fn find_impure_call_name(expr: &Expression) -> Option<String> {
        match expr {
            Expression::Call { callee, args } => {
                if let Some(name) = Self::identifier_name(callee.as_ref()) {
                    if name.ends_with('!') {
                        return Some(name);
                    }
                }
                Self::find_impure_call_name(callee.as_ref())
                    .or_else(|| args.iter().find_map(|arg| Self::find_impure_call_name(arg)))
            }
            Expression::Identifier(name) => {
                if name.ends_with('!') {
                    Some(name.clone())
                } else {
                    None
                }
            }
            Expression::Block(exprs) => exprs.iter().find_map(|e| Self::find_impure_call_name(e)),
            Expression::Lambda { body, .. } => Self::find_impure_call_name(body.as_ref()),
            Expression::Object(fields) => fields.iter().find_map(|f| match f {
                ObjectField::Field { value, .. } => Self::find_impure_call_name(value),
                ObjectField::Spread(expr) => Self::find_impure_call_name(expr),
            }),
            Expression::Spread(expr) => Self::find_impure_call_name(expr.as_ref()),
            Expression::List(elements) => {
                elements.iter().find_map(|e| Self::find_impure_call_name(e))
            }
            Expression::Binary { left, right, .. } => Self::find_impure_call_name(left.as_ref())
                .or_else(|| Self::find_impure_call_name(right.as_ref())),
            Expression::PropertyAccess { object, property } => {
                // Check if property name ends with '!' (impure method call)
                if property.ends_with('!') {
                    let obj_name = match object.as_ref() {
                        Expression::Identifier(name) => name.clone(),
                        _ => "<object>".to_string(),
                    };
                    Some(format!("{}.{}", obj_name, property))
                } else {
                    Self::find_impure_call_name(object.as_ref())
                }
            }
            Expression::String(template) => template.segments.iter().find_map(|s| {
                if let StringSegment::Expr(e) = s {
                    Self::find_impure_call_name(e)
                } else {
                    None
                }
            }),
            _ => None,
        }
    }

    fn identifier_name(expr: &Expression) -> Option<String> {
        match expr {
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn returns_boolean(expr: &Expression) -> bool {
        match expr {
            Expression::Boolean(_) => true,
            Expression::Binary { op, .. } => {
                matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::LessThan
                        | BinaryOperator::LessThanEq
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::GreaterThanEq
                        | BinaryOperator::And
                        | BinaryOperator::Or
                )
            }
            Expression::Call { callee, .. } => {
                if let Some(name) = Self::identifier_name(callee.as_ref()) {
                    name.ends_with('?')
                } else {
                    false
                }
            }
            Expression::Block(exprs) => exprs
                .last()
                .map(|e| Self::returns_boolean(e))
                .unwrap_or(false),
            _ => false,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fip-lint <file.fip|directory>");
        eprintln!("       fip-lint <file.fip>        Lint a single file");
        eprintln!("       fip-lint <directory>        Lint all .fip files recursively");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    let has_errors = if path.is_dir() {
        lint_directory(&path)
    } else if path.is_file() {
        let error_count = lint_file(&path);
        error_count > 0
    } else {
        eprintln!(
            "Error: Path '{}' is neither a file nor a directory",
            path.display()
        );
        std::process::exit(1);
    };

    if has_errors {
        std::process::exit(1);
    }
}

fn lint_file(file_path: &Path) -> usize {
    let file_path_str = file_path.to_string_lossy();
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("Error reading file: {}", e);
            let fake_error = vec![LintError {
                line: 1,
                column: 1,
                message: error_msg,
                severity: Severity::Error,
            }];
            print_file_status(&file_path_str, 1, &fake_error);
            return 1;
        }
    };

    let file_path_buf = file_path.to_path_buf();
    let tokens = match Lexer::new(&source).lex() {
        Ok(t) => t,
        Err(e) => {
            let error_msg = format!("Lexer error: {}", e);
            let fake_error = vec![LintError {
                line: 1,
                column: 1,
                message: error_msg,
                severity: Severity::Error,
            }];
            print_file_status(&file_path_str, 1, &fake_error);
            return 1;
        }
    };

    let mut parser = Parser::with_source_and_file(tokens, source.clone(), file_path_buf.clone());
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            // Extract location from parser error and format it properly
            let (line, msg) = match &e {
                LangError::Parser(msg, location) => {
                    if let Some(loc) = location {
                        (loc.line, msg.clone())
                    } else {
                        (1, msg.clone())
                    }
                }
                LangError::Lexer(msg, location) => {
                    if let Some(loc) = location {
                        (loc.line, msg.clone())
                    } else {
                        (1, msg.clone())
                    }
                }
                _ => (1, format!("{}", e)),
            };
            let fake_error = vec![LintError {
                line,
                column: 1,
                message: msg,
                severity: Severity::Error,
            }];
            print_file_status(&file_path_str, 1, &fake_error);
            return 1;
        }
    };

    let mut linter = Linter::new(source);
    let errors = linter.lint(&program);

    let error_count = errors
        .iter()
        .filter(|e| e.severity == Severity::Error)
        .count();

    print_file_status(&file_path_str, error_count, &errors);
    error_count
}

fn print_file_status(file_path: &str, error_count: usize, errors: &[LintError]) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    if error_count == 0 {
        // Print green tick + "ok" before filename
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
        let _ = write!(stdout, "✓ ok ");
        let _ = stdout.reset();
        let _ = writeln!(stdout, "{}", file_path);
    } else {
        // Print red "!" + filename on new line
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
        let _ = write!(stdout, "! ");
        let _ = stdout.reset();
        let _ = writeln!(stdout, "{}", file_path);

        // Print each error on its own line
        for error in errors.iter().filter(|e| e.severity == Severity::Error) {
            let _ = writeln!(stdout, "  row: {}: {}", error.line, error.message);
        }
    }

    let _ = stdout.reset();
}

fn lint_directory(dir_path: &Path) -> bool {
    let mut has_errors = false;
    let mut files_linted = 0;

    for entry in walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("fip") {
            files_linted += 1;
            let error_count = lint_file(path);
            if error_count > 0 {
                has_errors = true;
            }
        }
    }

    if files_linted == 0 {
        eprintln!("No .fip files found in {}", dir_path.display());
        return false;
    }

    has_errors
}
