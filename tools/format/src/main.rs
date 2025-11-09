use std::{env, fs, io};

use fippli_lang::ast::{
    BinaryOperator, Expression, Function, Program, Statement, StringSegment, UseStatement,
};
use fippli_lang::lexer::Lexer;
use fippli_lang::parser::Parser;

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
            Statement::Assignment { name, expr } => {
                format!("{}: {}", name, self.format_expression(expr))
            }
            Statement::Function(func) => self.format_function(func),
            Statement::Expression(expr) => self.format_expression(expr),
            Statement::Use(use_stmt) => self.format_use_statement(use_stmt),
            Statement::Export(export) => format!("export {}", export.name),
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
                    .map(|f| {
                        format!(
                            "{}{}: {}",
                            self.indent(),
                            f.name,
                            self.format_expression(&f.value)
                        )
                    })
                    .collect();
                self.indent_level = old_indent;
                format!("{{\n{}\n{}}}", formatted.join(",\n"), self.indent())
            }
            Expression::List(elements) => {
                if elements.is_empty() {
                    return "[]".to_string();
                }
                let formatted: Vec<String> =
                    elements.iter().map(|e| self.format_expression(e)).collect();
                format!("[{}]", formatted.join(", "))
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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fip-format <file.fip> [--write]");
        eprintln!("  --write: Write formatted output back to file (default: print to stdout)");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let write_mode = args.contains(&"--write".to_string()) || args.contains(&"-w".to_string());

    let source = fs::read_to_string(file_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read file: {}", e)))?;

    let tokens = Lexer::new(&source)
        .lex()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Parse error: {}", e)))?;

    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Parse error: {}", e)))?;

    let mut formatter = Formatter::new();
    let formatted = formatter.format_program(&program);

    if write_mode {
        fs::write(file_path, formatted).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to write file: {}", e))
        })?;
        println!("Formatted: {}", file_path);
    } else {
        print!("{}", formatted);
    }

    Ok(())
}
