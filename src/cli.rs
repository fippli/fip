use std::{
    env, fs,
    path::{Path, PathBuf},
};

use fippli_lang::ast::{
    BinaryOperator, Expression, Function, ObjectField, ObjectPatternField, Pattern, Program,
    Statement, StringSegment, UseStatement,
};
use fippli_lang::error::LangError;
use fippli_lang::interpreter::Interpreter;
use fippli_lang::lexer::Lexer;
use fippli_lang::parser::Parser as FipParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let result = match command.as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "version" | "--version" | "-v" => {
            print_version();
            Ok(())
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: 'run' command requires a file argument");
                eprintln!("Usage: fip run <file.fip>");
                std::process::exit(1);
            }
            run_command(&args[2])
        }
        "format" => {
            if args.len() < 3 {
                eprintln!("Error: 'format' command requires a file argument");
                eprintln!("Usage: fip format <file.fip> [--write]");
                std::process::exit(1);
            }
            let write = args.contains(&"--write".to_string()) || args.contains(&"-w".to_string());
            format_command(&args[2], write)
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!("FIP (Functional Intuitive Programming) language tool");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  fip run <file.fip>        Run a FIP program");
    eprintln!("  fip format <file.fip>     Format a FIP source file (prints to stdout)");
    eprintln!("  fip format <file.fip> -w  Format a FIP source file (writes to file)");
    eprintln!("  fip help                  Show this help message");
    eprintln!("  fip version               Show version information");
}

fn print_version() {
    println!("fip {}", env!("CARGO_PKG_VERSION"));
}

fn run_command(file: &str) -> Result<(), LangError> {
    let source_path = Path::new(file);
    if !source_path.exists() {
        return Err(LangError::Runtime(
            format!("Source file '{}' not found", file),
            None,
        ));
    }

    let source = fs::read_to_string(source_path)?;
    let tokens =
        Lexer::with_source_and_file(&source, source.clone(), source_path.to_path_buf()).lex()?;
    let mut parser =
        FipParser::with_source_and_file(tokens, source.clone(), source_path.to_path_buf());
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

fn format_command(file: &str, write: bool) -> Result<(), LangError> {
    let source = fs::read_to_string(file)
        .map_err(|e| LangError::Runtime(format!("Failed to read file: {}", e), None))?;

    let tokens = Lexer::with_source_and_file(&source, source.clone(), PathBuf::from(file))
        .lex()
        .map_err(|e| LangError::Runtime(format!("Parse error: {}", e), None))?;

    let mut parser = FipParser::with_source_and_file(tokens, source.clone(), PathBuf::from(file));
    let program = parser
        .parse_program()
        .map_err(|e| LangError::Runtime(format!("Parse error: {}", e), None))?;

    let mut formatter = Formatter::new();
    let formatted = formatter.format_program(&program);

    if write {
        fs::write(file, formatted)
            .map_err(|e| LangError::Runtime(format!("Failed to write file: {}", e), None))?;
        println!("Formatted: {}", file);
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

// Formatter implementation (copied from tools/format)
struct Formatter {
    indent_level: usize,
    indent_size: usize,
}

impl Formatter {
    fn new() -> Self {
        Self {
            indent_level: 0,
            indent_size: 2,
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn format_program(&mut self, program: &Program) -> String {
        let mut output = Vec::new();

        for (i, stmt) in program.statements.iter().enumerate() {
            if i > 0 {
                output.push(String::new());
            }
            output.push(self.format_statement(stmt));
        }

        output.join("\n")
    }

    fn format_statement(&mut self, stmt: &Statement) -> String {
        match stmt {
            Statement::Assignment { pattern, expr } => {
                format!(
                    "{}: {}",
                    self.format_pattern(pattern),
                    self.format_expression(expr)
                )
            }
            Statement::Function(func) => self.format_function(func),
            Statement::Expression(expr) => self.format_expression(expr),
            Statement::Use(use_stmt) => self.format_use_statement(use_stmt),
            Statement::Export(export) => format!("export {}", export.name),
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Identifier(name) => name.clone(),
            Pattern::List(patterns) => {
                let formatted: Vec<String> =
                    patterns.iter().map(|p| self.format_pattern(p)).collect();
                format!("[{}]", formatted.join(", "))
            }
            Pattern::Object(fields) => {
                let formatted: Vec<String> = fields
                    .iter()
                    .map(|f| match f {
                        ObjectPatternField::Shorthand(name) => name.clone(),
                        ObjectPatternField::Field { name, pattern } => {
                            format!("{}: {}", name, self.format_pattern(pattern))
                        }
                    })
                    .collect();
                format!("{{ {} }}", formatted.join(", "))
            }
        }
    }

    fn format_function(&mut self, func: &Function) -> String {
        let notation = if func.impure {
            "!"
        } else if func.name.ends_with('?') {
            "?"
        } else {
            ""
        };

        let name = if func.impure {
            func.name.strip_suffix('!').unwrap_or(&func.name)
        } else if func.name.ends_with('?') {
            func.name.strip_suffix('?').unwrap_or(&func.name)
        } else {
            &func.name
        };

        let params_str = func.params.join(", ");
        let old_indent = self.indent_level;
        self.indent_level += 1;
        let body_str = self.format_expression_with_indent(&func.body);
        self.indent_level = old_indent;

        format!(
            "{}{}: ({}) {{\n{}\n}}",
            name, notation, params_str, body_str
        )
    }

    fn format_use_statement(&mut self, use_stmt: &UseStatement) -> String {
        match use_stmt {
            UseStatement::Single { name, module_path } => {
                format!("use {} from \"{}\"", name, module_path)
            }
            UseStatement::Namespace { alias, module_path } => {
                format!("use {} as \"{}\"", alias, module_path)
            }
            UseStatement::Selective { names, module_path } => {
                let names_str = names.join(", ");
                format!("use {{ {} }} from \"{}\"", names_str, module_path)
            }
        }
    }

    fn format_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Number(n) => n.to_string(),
            Expression::String(template) => self.format_string_template(template),
            Expression::Boolean(b) => b.to_string(),
            Expression::Null => "null".to_string(),
            Expression::Identifier(name) => name.clone(),
            Expression::Block(exprs) => {
                if exprs.is_empty() {
                    return "{}".to_string();
                }
                let old_indent = self.indent_level;
                self.indent_level += 1;
                let formatted: Vec<String> = exprs
                    .iter()
                    .map(|e| format!("{}{}", self.indent(), self.format_expression(e)))
                    .collect();
                self.indent_level = old_indent;
                format!("{{\n{}\n{}}}", formatted.join("\n"), self.indent())
            }
            Expression::Lambda {
                params,
                body,
                impure,
            } => {
                let notation = if *impure { "!" } else { "" };
                let params_str = params.join(", ");
                let body_str = self.format_lambda_body(body);
                format!("({}){} {}", params_str, notation, body_str)
            }
            Expression::Object(fields) => {
                if fields.is_empty() {
                    return "{}".to_string();
                }
                let old_indent = self.indent_level;
                self.indent_level += 1;
                let formatted: Vec<String> = fields
                    .iter()
                    .map(|f| match f {
                        ObjectField::Field { name, value } => {
                            format!(
                                "{}{}: {}",
                                self.indent(),
                                name,
                                self.format_expression(value)
                            )
                        }
                        ObjectField::Spread(expr) => {
                            format!("{}...{}", self.indent(), self.format_expression(expr))
                        }
                    })
                    .collect();
                self.indent_level = old_indent;
                format!("{{\n{}\n{}}}", formatted.join(",\n"), self.indent())
            }
            Expression::List(elements) => {
                if elements.is_empty() {
                    return "[]".to_string();
                }
                let formatted: Vec<String> = elements
                    .iter()
                    .map(|e| match e {
                        Expression::Spread(expr) => {
                            format!("...{}", self.format_expression(expr.as_ref()))
                        }
                        other => self.format_expression(other),
                    })
                    .collect();
                format!("[{}]", formatted.join(", "))
            }
            Expression::Spread(expr) => {
                format!("...{}", self.format_expression(expr.as_ref()))
            }
            Expression::Call { callee, args } => {
                let callee_str = self.format_expression(callee);
                let args_str: Vec<String> =
                    args.iter().map(|a| self.format_expression(a)).collect();
                format!("{}({})", callee_str, args_str.join(", "))
            }
            Expression::PropertyAccess { object, property } => {
                format!("{}.{}", self.format_expression(object), property)
            }
            Expression::Binary { left, op, right } => {
                let left_str = self.format_expression(left);
                let right_str = self.format_expression(right);
                let op_str = match op {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Sub => "-",
                    BinaryOperator::Mul => "*",
                    BinaryOperator::Div => "/",
                    BinaryOperator::Eq => "=",
                    BinaryOperator::NotEq => "!=",
                    BinaryOperator::LessThan => "<",
                    BinaryOperator::LessThanEq => "<=",
                    BinaryOperator::GreaterThan => ">",
                    BinaryOperator::GreaterThanEq => ">=",
                    BinaryOperator::And => "&",
                    BinaryOperator::Or => "|",
                };
                format!("{} {} {}", left_str, op_str, right_str)
            }
        }
    }

    fn format_lambda_body(&mut self, body: &Expression) -> String {
        match body {
            Expression::Block(exprs) => {
                if exprs.is_empty() {
                    return "{}".to_string();
                }
                // Check if body is simple (single expression, not too complex)
                if exprs.len() == 1 && self.is_simple_expression(&exprs[0]) {
                    let body_str = self.format_expression(&exprs[0]);
                    format!("{{ {} }}", body_str)
                } else {
                    let old_indent = self.indent_level;
                    self.indent_level += 1;
                    let formatted: Vec<String> = exprs
                        .iter()
                        .map(|e| format!("{}{}", self.indent(), self.format_expression(e)))
                        .collect();
                    self.indent_level = old_indent;
                    format!("{{\n{}\n{}}}", formatted.join("\n"), self.indent())
                }
            }
            _ => {
                let body_str = self.format_expression(body);
                format!("{{ {} }}", body_str)
            }
        }
    }

    fn is_simple_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Number(_)
            | Expression::String(_)
            | Expression::Boolean(_)
            | Expression::Null
            | Expression::Identifier(_) => true,
            Expression::Binary { left, right, .. } => {
                self.is_simple_expression(left) && self.is_simple_expression(right)
            }
            Expression::PropertyAccess { object, .. } => {
                matches!(**object, Expression::Identifier(_))
            }
            Expression::Call { callee, args } => {
                matches!(**callee, Expression::Identifier(_))
                    && args.len() <= 2
                    && args.iter().all(|a| self.is_simple_expression(a))
            }
            _ => false,
        }
    }

    fn format_expression_with_indent(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Block(exprs) => {
                if exprs.is_empty() {
                    return format!("{}", self.indent());
                }
                let formatted: Vec<String> = exprs
                    .iter()
                    .map(|e| format!("{}{}", self.indent(), self.format_expression(e)))
                    .collect();
                formatted.join("\n")
            }
            _ => {
                format!("{}{}", self.indent(), self.format_expression(expr))
            }
        }
    }

    fn format_string_template(&self, template: &fippli_lang::ast::StringTemplate) -> String {
        let mut result = String::from("\"");
        for segment in &template.segments {
            match segment {
                StringSegment::Literal(s) => {
                    // Escape special characters
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    result.push_str(&escaped);
                }
                StringSegment::Expr(expr) => {
                    result.push('<');
                    result.push_str(&self.format_expression_inline(expr));
                    result.push('>');
                }
            }
        }
        result.push('"');
        result
    }

    fn format_expression_inline(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(name) => name.clone(),
            Expression::PropertyAccess { object, property } => {
                format!("{}.{}", self.format_expression_inline(object), property)
            }
            _ => {
                // For complex expressions, just format normally
                let mut formatter = Formatter::new();
                formatter.format_expression(expr)
            }
        }
    }
}
