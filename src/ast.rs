use crate::lexer::TokenContentType;

pub struct Literal {
    content: TokenContentType,
}

impl Literal {
    pub fn from(content: TokenContentType) -> Literal {
        Literal { content }
    }
}

pub enum Ast {
    Literal(Literal),
}
