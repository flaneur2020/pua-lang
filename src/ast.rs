#[derive(PartialEq, Debug)]
pub struct Ident(pub String);

#[derive(PartialEq, Debug)]
pub enum Prefix {
    Plus,
    Minus,
    Not,
}

#[derive(PartialEq, Debug)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
    Equal,
    NotEqual,
    GreaterThanEqual,
    GreaterThan,
    LessThanEqual,
    LessThan,
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Ident(Ident),
    Literal(Literal),
    Prefix(Prefix, Box<Expr>),
    Infix(Infix, Box<Expr>, Box<Expr>),
    If {
        cond: Box<Expr>,
        consequence: BlockStmt,
        alternative: Option<BlockStmt>,
    },
}

#[derive(PartialEq, Debug)]
pub enum Literal {
    Int(i64),
    Bool(bool),
}

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Let(Ident, Expr),
    Return(Expr),
    Expr(Expr),
}

pub type BlockStmt = Vec<Stmt>;

pub type Program = BlockStmt;

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(x)
}
