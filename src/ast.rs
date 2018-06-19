pub type BlockStmt = Vec<Stmt>;

pub type Program = BlockStmt;

#[derive(PartialEq, Debug)]
pub struct Ident(pub String);

#[derive(PartialEq, Debug)]
pub enum Prefix {
    Plus,
    Minus,
    Not,
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Ident(Ident),
    Prefix(Prefix, Box<Expr>),
    Infix(Prefix, Box<Expr>),
}

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Let(Ident, Expr),
    Return(Expr),
}
