use crate::{
    ast::{
        BinaryOperator, ExportStatement, Expression, Function, ObjectField, ObjectPatternField,
        Pattern, Program, Statement, StringSegment, StringTemplate, UseStatement,
    },
    error::{byte_offset_to_line, LangError, LangResult, Location},
    lexer::{Lexer, Token, TokenKind},
};
use std::path::PathBuf;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    source: String,
    file_path: PathBuf,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            source: String::new(),
            file_path: PathBuf::from("<unknown>"),
        }
    }

    pub fn with_source_and_file(tokens: Vec<Token>, source: String, file_path: PathBuf) -> Self {
        Self {
            tokens,
            current: 0,
            source,
            file_path,
        }
    }

    fn error_with_location(&self, msg: String) -> LangError {
        let location = if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            let line = byte_offset_to_line(&self.source, token.span.start);
            Some(Location::new(self.file_path.clone(), line))
        } else if !self.tokens.is_empty() {
            let last_token = &self.tokens[self.tokens.len() - 1];
            let line = byte_offset_to_line(&self.source, last_token.span.end);
            Some(Location::new(self.file_path.clone(), line))
        } else {
            None
        };
        LangError::Parser(msg, location)
    }

    pub fn parse_program(&mut self) -> LangResult<Program> {
        let mut statements = Vec::new();
        let mut statement_starts = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            let start_pos = self.current_token().span.start;
            statement_starts.push(start_pos);
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        let program = Program { statements };

        // Validate variable restrictions with statement start positions
        self.validate_program(&program, &statement_starts)?;

        Ok(program)
    }

    fn validate_program(&self, program: &Program, statement_starts: &[usize]) -> LangResult<()> {
        use std::collections::HashSet;

        let mut defined_names = HashSet::new();

        for (statement_index, statement) in program.statements.iter().enumerate() {
            let statement_start = statement_starts.get(statement_index).copied().unwrap_or(0);
            match statement {
                Statement::Assignment { pattern, .. } => {
                    // Validate pattern and collect all identifiers
                    let identifiers = self.collect_pattern_identifiers(pattern)?;
                    for name in &identifiers {
                        // Validate kebab-case
                        self.validate_kebab_case(name)?;

                        // Check for duplicate binding
                        if defined_names.contains(name) {
                            // Find the identifier in this statement
                            let error_location =
                                self.find_identifier_in_statement(statement_start, name);
                            return Err(self.error_at_location(
                                error_location,
                                format!("Mutation error: trying to mutate binding {}", name),
                            ));
                        }
                        defined_names.insert(name.clone());
                    }
                }
                Statement::Function(func) => {
                    // Validate kebab-case for function name
                    self.validate_kebab_case(&func.name)?;

                    // Check for duplicate binding
                    if defined_names.contains(&func.name) {
                        let error_location =
                            self.find_identifier_in_statement(statement_start, &func.name);
                        return Err(self.error_at_location(
                            error_location,
                            format!("Cannot redefine immutable binding '{}'", func.name),
                        ));
                    }
                    defined_names.insert(func.name.clone());

                    // Validate parameter names (they should also be kebab-case)
                    for param in &func.params {
                        self.validate_kebab_case(param)?;
                    }
                }
                Statement::Use(use_stmt) => match use_stmt {
                    UseStatement::Single { name, .. } => {
                        self.validate_kebab_case(name)?;
                        if defined_names.contains(name) {
                            let error_location =
                                self.find_identifier_in_statement(statement_start, name);
                            return Err(self.error_at_location(
                                error_location,
                                format!("Cannot redefine immutable binding '{}'", name),
                            ));
                        }
                        defined_names.insert(name.clone());
                    }
                    UseStatement::Namespace { alias, .. } => {
                        self.validate_kebab_case(alias)?;
                        if defined_names.contains(alias) {
                            let error_location =
                                self.find_identifier_in_statement(statement_start, alias);
                            return Err(self.error_at_location(
                                error_location,
                                format!("Cannot redefine immutable binding '{}'", alias),
                            ));
                        }
                        defined_names.insert(alias.clone());
                    }
                    UseStatement::Selective { names, .. } => {
                        for name in names {
                            self.validate_kebab_case(name)?;
                            if defined_names.contains(name) {
                                let error_location =
                                    self.find_identifier_in_statement(statement_start, name);
                                return Err(self.error_at_location(
                                    error_location,
                                    format!("Cannot redefine immutable binding '{}'", name),
                                ));
                            }
                            defined_names.insert(name.clone());
                        }
                    }
                },
                Statement::Export(export) => {
                    // Exports don't create bindings, but validate the name format
                    self.validate_kebab_case(&export.name)?;
                }
                Statement::Expression(_) => {
                    // Expressions don't create bindings
                }
            }
        }

        Ok(())
    }

    fn find_identifier_in_statement(&self, statement_start: usize, name: &str) -> usize {
        // Find the token that starts at or after statement_start
        let mut token_index = 0;
        while token_index < self.tokens.len() {
            if self.tokens[token_index].span.start >= statement_start {
                break;
            }
            token_index += 1;
        }

        // Search for the identifier in this statement
        while token_index < self.tokens.len() {
            let token = &self.tokens[token_index];
            match &token.kind {
                TokenKind::Identifier(id) if id == name => {
                    return token.span.start;
                }
                TokenKind::Newline => {
                    // End of statement (but continue to next statement start if we haven't found it)
                    let next_token_index = token_index + 1;
                    if next_token_index < self.tokens.len() {
                        // Check if next statement starts (non-newline token)
                        if !matches!(self.tokens[next_token_index].kind, TokenKind::Newline) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ => {}
            }
            token_index += 1;
        }

        // Fallback: use statement start
        statement_start
    }

    fn error_at_location(&self, byte_offset: usize, msg: String) -> LangError {
        let line = byte_offset_to_line(&self.source, byte_offset);
        let location = Some(Location::new(self.file_path.clone(), line));
        LangError::Parser(msg, location)
    }

    fn collect_pattern_identifiers(&self, pattern: &Pattern) -> LangResult<Vec<String>> {
        let mut identifiers = Vec::new();
        match pattern {
            Pattern::Identifier(name) => {
                identifiers.push(name.clone());
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    identifiers.extend(self.collect_pattern_identifiers(p)?);
                }
            }
            Pattern::Object(fields) => {
                for field in fields {
                    match field {
                        ObjectPatternField::Shorthand(name) => {
                            identifiers.push(name.clone());
                        }
                        ObjectPatternField::Field { name: _, pattern } => {
                            // The field name itself doesn't create a binding, but the pattern does
                            identifiers.extend(self.collect_pattern_identifiers(pattern)?);
                        }
                    }
                }
            }
        }
        Ok(identifiers)
    }

    fn validate_kebab_case(&self, name: &str) -> LangResult<()> {
        // Check if name is empty
        if name.is_empty() {
            return Err(self.error_with_location("Identifier name cannot be empty".to_string()));
        }

        // Handle function suffixes (! and ?) - strip them for validation
        let base_name = if name.ends_with('!') || name.ends_with('?') {
            &name[..name.len() - 1]
        } else {
            name
        };

        // After stripping suffix, base name cannot be empty
        if base_name.is_empty() {
            return Err(self.error_with_location(format!(
                "Identifier '{}' must have a name before the suffix",
                name
            )));
        }

        // Check if base name starts or ends with hyphen
        if base_name.starts_with('-') || base_name.ends_with('-') {
            return Err(self.error_with_location(format!(
                "Identifier '{}' cannot start or end with a hyphen",
                name
            )));
        }

        // Check for consecutive hyphens
        if base_name.contains("--") {
            return Err(self.error_with_location(format!(
                "Identifier '{}' cannot contain consecutive hyphens",
                name
            )));
        }

        // Check that all characters are lowercase letters, digits, or hyphens
        // and that it follows kebab-case pattern
        let mut chars = base_name.chars().peekable();
        let mut has_letter = false;

        while let Some(ch) = chars.next() {
            match ch {
                'a'..='z' => {
                    has_letter = true;
                }
                '0'..='9' => {
                    // Digits are allowed but name must start with a letter
                    if !has_letter {
                        return Err(self.error_with_location(format!(
                            "Identifier '{}' must start with a lowercase letter",
                            name
                        )));
                    }
                }
                '-' => {
                    // Hyphens are allowed but must be followed by a letter or digit
                    if let Some(&next) = chars.peek() {
                        if !matches!(next, 'a'..='z' | '0'..='9') {
                            return Err(self.error_with_location(
                                format!("Identifier '{}' must have a lowercase letter or digit after each hyphen", name)
                            ));
                        }
                    } else {
                        // Hyphen at end is already caught above
                        return Err(self.error_with_location(format!(
                            "Identifier '{}' cannot end with a hyphen",
                            name
                        )));
                    }
                }
                '_' => {
                    // Underscores are not allowed in kebab-case
                    return Err(self.error_with_location(
                        format!("Identifier '{}' contains underscore. Identifiers must use kebab-case (lowercase letters, digits, and hyphens, not underscores)", name)
                    ));
                }
                _ => {
                    return Err(self.error_with_location(
                        format!("Identifier '{}' contains invalid character '{}'. Identifiers must use kebab-case (lowercase letters, digits, and hyphens)", name, ch)
                    ));
                }
            }
        }

        // Name must contain at least one letter
        if !has_letter {
            return Err(self.error_with_location(format!(
                "Identifier '{}' must contain at least one letter",
                name
            )));
        }

        Ok(())
    }

    fn parse_statement(&mut self) -> LangResult<Statement> {
        self.skip_newlines();
        let start_index = self.current;

        // Check for 'use' statement
        if let TokenKind::Identifier(ref name) = self.current_kind() {
            if name == "use" {
                return self.parse_use_statement();
            }
            if name == "export" {
                return self.parse_export_statement();
            }
        }

        // Try to parse a pattern (identifier or list pattern)
        let pattern = match self.try_parse_pattern() {
            Some(pattern) => pattern,
            None => {
                let expr = self.parse_expression()?;
                return Ok(Statement::Expression(expr));
            }
        };

        if matches!(self.current_kind(), TokenKind::Colon) {
            self.advance();
            self.skip_newlines();
            let expr_start = self.current;

            // Check if this is a function definition
            // Functions must have Pattern::Identifier
            if let Pattern::Identifier(ref name) = pattern {
                let is_potential_function = matches!(self.current_kind(), TokenKind::LParen)
                    && matches!(
                        self.peek_non_newline_kind(self.current + 1),
                        Some(TokenKind::Identifier(_)) | Some(TokenKind::RParen)
                    );

                if is_potential_function {
                    let params_start = self.current;
                    self.advance(); // consume '('
                    self.skip_newlines();
                    let params_result = self.parse_parameter_list();

                    match params_result {
                        Ok(params) => {
                            self.skip_newlines();
                            match self.expect(TokenKind::RParen, "Expected ')' after parameters") {
                                Ok(()) => {
                                    self.skip_newlines();
                                    if matches!(self.current_kind(), TokenKind::LBrace) {
                                        self.advance();
                                        let body_expressions = self.parse_block_contents()?;
                                        self.expect(
                                            TokenKind::RBrace,
                                            "Expected '}' after function body",
                                        )?;
                                        let impure = name.ends_with('!');
                                        return Ok(Statement::Function(Function {
                                            name: name.clone(),
                                            params,
                                            body: Expression::Block(body_expressions),
                                            impure,
                                        }));
                                    } else {
                                        self.current = expr_start;
                                    }
                                }
                                Err(err) => return Err(err),
                            }
                        }
                        Err(err) => {
                            if self.current != params_start {
                                return Err(err);
                            }
                            self.current = expr_start;
                        }
                    }
                }
            }

            self.current = expr_start;
            self.skip_newlines();
            let expr = self.parse_expression()?;
            return Ok(Statement::Assignment { pattern, expr });
        }

        self.current = start_index;
        let expr = self.parse_expression()?;
        Ok(Statement::Expression(expr))
    }

    fn try_parse_pattern(&mut self) -> Option<Pattern> {
        // Try to parse an object pattern { field, ... }
        if matches!(self.current_kind(), TokenKind::LBrace) {
            let brace_pos = self.current;
            self.advance();
            self.skip_newlines();

            let mut fields = Vec::new();

            // Handle empty object pattern {}
            if matches!(self.current_kind(), TokenKind::RBrace) {
                self.advance();
                return Some(Pattern::Object(fields));
            }

            loop {
                // Check for identifier (field name)
                let field_start = self.current;
                let field_name = match self.current_kind().clone() {
                    TokenKind::Identifier(name) => {
                        self.advance();
                        name
                    }
                    _ => {
                        // Not an object pattern, reset to before the brace
                        self.current = brace_pos;
                        return None;
                    }
                };

                self.skip_newlines();

                // Check if this is shorthand { name } or field { name: pattern }
                if matches!(self.current_kind(), TokenKind::Colon) {
                    // Field with nested pattern: { name: pattern }
                    self.advance();
                    self.skip_newlines();
                    match self.try_parse_pattern() {
                        Some(pattern) => {
                            fields.push(ObjectPatternField::Field {
                                name: field_name,
                                pattern,
                            });
                        }
                        None => {
                            // Not a valid pattern - check if it's a non-pattern token
                            // (like string literal, number, etc.) which means this is an object expression, not pattern
                            if matches!(
                                self.current_kind(),
                                TokenKind::StringLiteral(_)
                                    | TokenKind::Number(_)
                                    | TokenKind::Boolean(_)
                                    | TokenKind::Null
                                    | TokenKind::LParen
                                    | TokenKind::LBracket
                            ) {
                                // This is definitely an object expression, not a pattern
                                self.current = brace_pos;
                                return None;
                            }
                            // Otherwise, reset to before this field attempt
                            self.current = field_start;
                            break;
                        }
                    }
                } else {
                    // Shorthand: { name }
                    fields.push(ObjectPatternField::Shorthand(field_name));
                }

                self.skip_newlines();

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else {
                    break;
                }
            }

            if matches!(self.current_kind(), TokenKind::RBrace) {
                self.advance();
                return Some(Pattern::Object(fields));
            } else {
                // Reset if we didn't find closing brace
                self.current = brace_pos;
                return None;
            }
        }

        // Try to parse a list pattern [pattern, pattern, ...]
        if matches!(self.current_kind(), TokenKind::LBracket) {
            let bracket_pos = self.current;
            self.advance();
            self.skip_newlines();

            let mut patterns = Vec::new();

            // Handle empty list pattern []
            if matches!(self.current_kind(), TokenKind::RBracket) {
                self.advance();
                return Some(Pattern::List(patterns));
            }

            loop {
                match self.try_parse_pattern() {
                    Some(pattern) => {
                        patterns.push(pattern);
                        self.skip_newlines();
                        if matches!(self.current_kind(), TokenKind::Comma) {
                            self.advance();
                            self.skip_newlines();
                        } else {
                            break;
                        }
                    }
                    None => {
                        // If we can't parse a pattern, reset to before the bracket
                        self.current = bracket_pos;
                        return None;
                    }
                }
            }

            if matches!(self.current_kind(), TokenKind::RBracket) {
                self.advance();
                return Some(Pattern::List(patterns));
            } else {
                // Reset if we didn't find closing bracket
                self.current = bracket_pos;
                return None;
            }
        }

        // Try to parse an identifier pattern
        if let TokenKind::Identifier(name) = self.current_kind().clone() {
            self.advance();
            return Some(Pattern::Identifier(name));
        }

        None
    }

    fn parse_parameter_list(&mut self) -> LangResult<Vec<String>> {
        let mut params = Vec::new();
        self.skip_newlines();
        if matches!(self.current_kind(), TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let name = self.consume_identifier("Expected parameter name")?;
            if name.ends_with('!') {
                return Err(
                    self.error_with_location("Parameter names cannot end with '!'".to_string())
                );
            }
            // Validate kebab-case for parameter names
            self.validate_kebab_case(&name)?;
            params.push(name);

            self.skip_newlines();
            if matches!(self.current_kind(), TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_expression(&mut self) -> LangResult<Expression> {
        self.skip_newlines();
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> LangResult<Expression> {
        let mut left = self.parse_unary_expression()?;

        loop {
            self.skip_newlines();
            let precedence = if let Some(precedence) = self.current_precedence() {
                precedence
            } else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            let op = self.parse_operator()?;
            let next_min = precedence + 1;
            let right = self.parse_binary_expression(next_min)?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> LangResult<Expression> {
        self.skip_newlines();
        if matches!(self.current_kind(), TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary_expression()?;
            Ok(Expression::Binary {
                left: Box::new(Expression::Number(0)),
                op: BinaryOperator::Sub,
                right: Box::new(expr),
            })
        } else {
            self.parse_call_expression()
        }
    }

    fn parse_call_expression(&mut self) -> LangResult<Expression> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if matches!(self.current_kind(), TokenKind::Newline) {
                break;
            }
            if matches!(self.current_kind(), TokenKind::LParen) {
                self.advance();
                self.skip_newlines();
                let args = self.parse_argument_list()?;
                self.expect(TokenKind::RParen, "Expected ')' after arguments")?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if matches!(self.current_kind(), TokenKind::Dot) {
                self.advance();
                self.skip_newlines();
                let property = match self.current_kind().clone() {
                    TokenKind::Identifier(name) => {
                        self.advance();
                        name
                    }
                    TokenKind::Number(value) => {
                        if value < 0 {
                            return Err(self.error_with_location(
                                "List indices must be non-negative".to_string(),
                            ));
                        }
                        self.advance();
                        value.to_string()
                    }
                    _ => {
                        return Err(self.error_with_location(
                            "Expected property name or index after '.'".to_string(),
                        ))
                    }
                };
                expr = Expression::PropertyAccess {
                    object: Box::new(expr),
                    property,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> LangResult<Expression> {
        match self.current_kind().clone() {
            TokenKind::Number(value) => {
                self.advance();
                Ok(Expression::Number(value))
            }
            TokenKind::Boolean(value) => {
                self.advance();
                Ok(Expression::Boolean(value))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Null)
            }
            TokenKind::StringLiteral(raw) => {
                self.advance();
                let template = self.parse_string_template(&raw)?;
                Ok(Expression::String(template))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            TokenKind::LBrace => {
                self.advance();
                if let Some(object) = self.try_parse_object()? {
                    return Ok(object);
                }
                let expressions = self.parse_block_contents()?;
                self.expect(TokenKind::RBrace, "Expected '}' after block")?;
                Ok(Expression::Block(expressions))
            }
            TokenKind::LBracket => {
                self.advance();
                let elements = self.parse_list_elements()?;
                self.expect(TokenKind::RBracket, "Expected ']' after list")?;
                Ok(Expression::List(elements))
            }
            TokenKind::LParen => {
                if let Some(lambda) = self.try_parse_lambda()? {
                    return Ok(lambda);
                }
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            other => {
                Err(self.error_with_location(format!("Unexpected token {:?} in expression", other)))
            }
        }
    }

    fn parse_argument_list(&mut self) -> LangResult<Vec<Expression>> {
        let mut args = Vec::new();
        self.skip_newlines();
        if matches!(self.current_kind(), TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            self.skip_newlines();
            if matches!(self.current_kind(), TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_list_elements(&mut self) -> LangResult<Vec<Expression>> {
        let mut elements = Vec::new();
        self.skip_newlines();
        if matches!(self.current_kind(), TokenKind::RBracket) {
            return Ok(elements);
        }

        loop {
            // Check for spread operator
            if matches!(self.current_kind(), TokenKind::Spread) {
                self.advance();
                self.skip_newlines();
                let expr = self.parse_expression()?;
                elements.push(Expression::Spread(Box::new(expr)));
            } else {
                elements.push(self.parse_expression()?);
            }
            self.skip_newlines();
            if matches!(self.current_kind(), TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            } else {
                break;
            }
        }

        Ok(elements)
    }

    fn parse_operator(&mut self) -> LangResult<BinaryOperator> {
        let op = match self.current_kind() {
            TokenKind::Plus => BinaryOperator::Add,
            TokenKind::Minus => BinaryOperator::Sub,
            TokenKind::Star => BinaryOperator::Mul,
            TokenKind::Slash => BinaryOperator::Div,
            TokenKind::Equal => BinaryOperator::Eq,
            TokenKind::NotEqual => BinaryOperator::NotEq,
            TokenKind::LessThan => BinaryOperator::LessThan,
            TokenKind::LessThanEq => BinaryOperator::LessThanEq,
            TokenKind::GreaterThan => BinaryOperator::GreaterThan,
            TokenKind::GreaterThanEq => BinaryOperator::GreaterThanEq,
            TokenKind::Ampersand => BinaryOperator::And,
            TokenKind::Pipe => BinaryOperator::Or,
            other => {
                return Err(
                    self.error_with_location(format!("Expected operator but found {:?}", other))
                )
            }
        };
        self.advance();
        Ok(op)
    }

    fn current_precedence(&self) -> Option<u8> {
        match self.current_kind() {
            TokenKind::Pipe => Some(0),
            TokenKind::Ampersand => Some(1),
            TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::LessThan
            | TokenKind::LessThanEq
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEq => Some(2),
            TokenKind::Plus | TokenKind::Minus => Some(3),
            TokenKind::Star | TokenKind::Slash => Some(4),
            _ => None,
        }
    }

    fn parse_string_template(&self, raw: &str) -> LangResult<StringTemplate> {
        let mut segments = Vec::new();
        let mut current = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '<' {
                if !current.is_empty() {
                    segments.push(StringSegment::Literal(current.clone()));
                    current.clear();
                }
                let mut expr_content = String::new();
                let mut found_end = false;
                while let Some(inner) = chars.next() {
                    if inner == '>' {
                        found_end = true;
                        break;
                    } else {
                        expr_content.push(inner);
                    }
                }
                if !found_end {
                    return Err(self.error_with_location(
                        "Unterminated interpolation in string literal".to_string(),
                    ));
                }
                let expr = Self::parse_template_expression(expr_content.trim())?;
                segments.push(StringSegment::Expr(expr));
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            segments.push(StringSegment::Literal(current));
        }

        Ok(StringTemplate { segments })
    }

    fn parse_template_expression(src: &str) -> LangResult<Expression> {
        if src.is_empty() {
            return Err(LangError::Parser(
                "Interpolation expression cannot be empty".to_string(),
                None,
            ));
        }
        let tokens = Lexer::new(src).lex()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression()?;
        parser.skip_newlines();
        if !parser.is_at_end() {
            return Err(LangError::Parser(
                "Unexpected tokens after interpolation expression".to_string(),
                None,
            ));
        }
        Ok(expr)
    }

    fn consume_identifier(&mut self, msg: &str) -> LangResult<String> {
        if let TokenKind::Identifier(name) = self.current_kind().clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.error_with_location(msg.to_string()))
        }
    }

    fn expect(&mut self, expected: TokenKind, msg: &str) -> LangResult<()> {
        self.skip_newlines();
        if std::mem::discriminant(self.current_kind()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_with_location(msg.to_string()))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn current_kind(&self) -> &TokenKind {
        &self.current_token().kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.current_token()
    }

    fn skip_newlines(&mut self) {
        while !self.is_at_end() && matches!(self.current_kind(), TokenKind::Newline) {
            self.current += 1;
        }
    }

    fn peek_non_newline_kind(&self, mut index: usize) -> Option<TokenKind> {
        while index < self.tokens.len() {
            let kind = &self.tokens[index].kind;
            if matches!(kind, TokenKind::Newline) {
                index += 1;
                continue;
            }
            return Some(kind.clone());
        }
        None
    }

    fn try_parse_lambda(&mut self) -> LangResult<Option<Expression>> {
        let start = self.current;
        self.advance(); // consume '('
        self.skip_newlines();

        let mut params = Vec::new();
        if matches!(self.current_kind(), TokenKind::RParen) {
            self.advance();
        } else {
            loop {
                match self.current_kind().clone() {
                    TokenKind::Identifier(name) => {
                        if name.ends_with('!') {
                            return Err(self.error_with_location(
                                "Parameter names cannot end with '!'".to_string(),
                            ));
                        }
                        // Validate kebab-case for parameter names
                        self.validate_kebab_case(&name)?;
                        params.push(name);
                        self.advance();
                    }
                    _ => {
                        self.current = start;
                        return Ok(None);
                    }
                }
                self.skip_newlines();
                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
            if !matches!(self.current_kind(), TokenKind::RParen) {
                self.current = start;
                return Ok(None);
            }
            self.advance();
        }

        self.skip_newlines();

        // Check for impure (!) suffix after closing paren
        let mut impure = false;

        if matches!(self.current_kind(), TokenKind::Exclamation) {
            self.advance();
            impure = true;
            self.skip_newlines();
        } else if matches!(self.current_kind(), TokenKind::Question) {
            // Boolean notation (?) - for now we just skip it
            // Boolean validation happens at runtime based on return type
            self.advance();
            self.skip_newlines();
        }

        if !matches!(self.current_kind(), TokenKind::LBrace) {
            self.current = start;
            return Ok(None);
        }
        self.advance();
        let body_expressions = self.parse_block_contents()?;
        self.expect(TokenKind::RBrace, "Expected '}' after block")?;

        Ok(Some(Expression::Lambda {
            params,
            body: Box::new(Expression::Block(body_expressions)),
            impure,
        }))
    }

    fn try_parse_object(&mut self) -> LangResult<Option<Expression>> {
        let start = self.current;
        self.skip_newlines();

        if matches!(self.current_kind(), TokenKind::RBrace) {
            self.advance();
            return Ok(Some(Expression::Object(Vec::new())));
        }

        let mut fields = Vec::new();

        loop {
            // Check for spread operator
            if matches!(self.current_kind(), TokenKind::Spread) {
                self.advance();
                self.skip_newlines();
                let expr = self.parse_expression()?;
                fields.push(ObjectField::Spread(expr));
                self.skip_newlines();

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                    continue;
                } else {
                    break;
                }
            }

            let name = match self.current_kind().clone() {
                TokenKind::Identifier(name) => {
                    self.advance();
                    name
                }
                _ => {
                    self.current = start;
                    return Ok(None);
                }
            };

            self.skip_newlines();
            if !matches!(self.current_kind(), TokenKind::Colon) {
                self.current = start;
                return Ok(None);
            }
            self.advance();
            self.skip_newlines();
            let value = self.parse_expression()?;
            fields.push(ObjectField::Field { name, value });
            self.skip_newlines();

            if matches!(self.current_kind(), TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
                // Check if there's a trailing comma (next token is closing brace)
                if matches!(self.current_kind(), TokenKind::RBrace) {
                    break;
                }
                continue;
            }
            break;
        }

        if !matches!(self.current_kind(), TokenKind::RBrace) {
            self.current = start;
            return Ok(None);
        }
        self.advance();

        Ok(Some(Expression::Object(fields)))
    }

    fn parse_block_contents(&mut self) -> LangResult<Vec<Expression>> {
        let mut expressions = Vec::new();
        self.skip_newlines();

        while !matches!(self.current_kind(), TokenKind::RBrace) {
            if self.is_at_end() {
                return Err(self.error_with_location("Unterminated block expression".to_string()));
            }
            let expr = self.parse_expression()?;
            expressions.push(expr);
            self.skip_newlines();
        }

        Ok(expressions)
    }

    fn parse_use_statement(&mut self) -> LangResult<Statement> {
        self.advance(); // consume 'use'
        self.skip_newlines();

        // Check for selective import: use {name1, name2} from "..."
        if matches!(self.current_kind(), TokenKind::LBrace) {
            self.advance(); // consume '{'
            self.skip_newlines();
            let mut names = Vec::new();

            loop {
                let name = self.consume_identifier("Expected identifier in selective import")?;
                names.push(name);
                self.skip_newlines();

                if matches!(self.current_kind(), TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else {
                    break;
                }
            }

            self.expect(
                TokenKind::RBrace,
                "Expected '}' after selective import list",
            )?;
            self.skip_newlines();
            if let TokenKind::Identifier(ref name) = self.current_kind() {
                if name != "from" {
                    return Err(
                        self.error_with_location("Expected 'from' after import list".to_string())
                    );
                }
                self.advance();
            } else {
                return Err(
                    self.error_with_location("Expected 'from' after import list".to_string())
                );
            }
            self.skip_newlines();
            let module_path = self.parse_module_path()?;
            return Ok(Statement::Use(UseStatement::Selective {
                names,
                module_path,
            }));
        }

        // Check for namespace import: use name as alias from "..."
        let first_name = self.consume_identifier("Expected identifier after 'use'")?;
        self.skip_newlines();

        if let TokenKind::Identifier(ref name) = self.current_kind() {
            if name == "as" {
                self.advance(); // consume 'as'
                self.skip_newlines();
                let alias = self.consume_identifier("Expected alias name after 'as'")?;
                self.skip_newlines();
                if let TokenKind::Identifier(ref name) = self.current_kind() {
                    if name != "from" {
                        return Err(
                            self.error_with_location("Expected 'from' after alias".to_string())
                        );
                    }
                    self.advance();
                } else {
                    return Err(self.error_with_location("Expected 'from' after alias".to_string()));
                }
                self.skip_newlines();
                let module_path = self.parse_module_path()?;
                return Ok(Statement::Use(UseStatement::Namespace {
                    alias,
                    module_path,
                }));
            }
        }

        // Single import: use name from "..."
        if let TokenKind::Identifier(ref name) = self.current_kind() {
            if name == "from" {
                self.advance(); // consume 'from'
                self.skip_newlines();
                let module_path = self.parse_module_path()?;
                return Ok(Statement::Use(UseStatement::Single {
                    name: first_name,
                    module_path,
                }));
            }
        }

        Err(self.error_with_location("Expected 'from' after import name".to_string()))
    }

    fn parse_export_statement(&mut self) -> LangResult<Statement> {
        self.advance(); // consume 'export'
        self.skip_newlines();
        let name = self.consume_identifier("Expected identifier after 'export'")?;
        Ok(Statement::Export(ExportStatement { name }))
    }

    fn parse_module_path(&mut self) -> LangResult<String> {
        match self.current_kind().clone() {
            TokenKind::StringLiteral(path) => {
                self.advance();
                Ok(path)
            }
            _ => {
                Err(self.error_with_location("Expected string literal for module path".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_lambda_expression() {
        let source = "(value) { trace!(\"after\", value) }";
        let tokens = Lexer::new(source)
            .lex()
            .expect("lexing should succeed for lambda expression");
        let mut parser = Parser::new(tokens);
        let expr = parser
            .parse_expression()
            .expect("parsing should succeed for lambda expression");
        match expr {
            Expression::Lambda { params, .. } => assert_eq!(params, vec!["value"]),
            other => panic!("expected lambda, got {:?}", other),
        }
    }
}
