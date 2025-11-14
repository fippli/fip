#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    List(Vec<Pattern>),
    Object(Vec<ObjectPatternField>),
}

#[derive(Debug, Clone)]
pub enum ObjectPatternField {
    Shorthand(String),                        // { name } - shorthand for { name: name }
    Field { name: String, pattern: Pattern }, // { name: pattern } - nested destructuring
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { pattern: Pattern, expr: Expression },
    Function(Function),
    Expression(Expression),
    Use(UseStatement),
    Export(ExportStatement),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expression,
    pub impure: bool,
    pub async_fn: bool,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    String(StringTemplate),
    Boolean(bool),
    Null,
    Identifier(String),
    Block(Vec<Expression>),
    Lambda {
        params: Vec<String>,
        body: Box<Expression>,
        impure: bool,
        async_fn: bool,
    },
    Await(Box<Expression>),
    Object(Vec<ObjectField>),
    List(Vec<Expression>),
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    Spread(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum ObjectField {
    Field { name: String, value: Expression },
    Spread(Expression),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct StringTemplate {
    pub segments: Vec<StringSegment>,
}

#[derive(Debug, Clone)]
pub enum StringSegment {
    Literal(String),
    Expr(Expression),
}

#[derive(Debug, Clone)]
pub enum UseStatement {
    Single {
        name: String,
        module_path: String,
    },
    Namespace {
        alias: String,
        module_path: String,
    },
    Selective {
        names: Vec<String>,
        module_path: String,
    },
}

#[derive(Debug, Clone)]
pub struct ExportStatement {
    pub name: String,
}
