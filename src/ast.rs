#[derive(Debug)]
pub enum Lit {
    Bool(bool),
    Int(i32),
    UInt(u32),
}

#[derive(Debug)]
pub enum UnOp {
    Neg,
}

#[derive(Debug)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
}

#[derive(Debug)]
pub enum Expr {
    Lit(Lit),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}
