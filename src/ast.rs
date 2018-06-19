pub type BlockStmt = Vec<Stmt>;

pub type Program = BlockStmt;

#[derive(PartialEq, Debug)]
pub struct Ident(pub String);

#[derive(PartialEq, Debug)]
pub enum Expr {
    IdentExpr(Ident),
}

#[derive(PartialEq, Debug)]
pub enum Stmt {
    LetStmt(Ident, Expr),
}
