use crate::lexer::TokenContentType;

pub struct Literal {
    content: TokenContentType,
}

impl Literal {
    pub fn from(content: TokenContentType) -> Literal {
        Literal { content }
    }
}

pub struct Array {
    // TODO: fix this vec type lmao
    content: Vec<Ast>,
}

impl Array {
    pub fn from(content: Vec<Ast>) -> Array {
        Array { content }
    }
}

pub enum Ast {
    Literal(Literal),
    Array(Array),
}
