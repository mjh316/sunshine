use crate::lexer::{TokenContentType, TokenType};

#[derive(Debug, Clone)]
pub struct Literal {
    content: TokenContentType,
}

impl Literal {
    pub fn from(content: TokenContentType) -> Literal {
        Literal { content }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    // TODO: fix this vec type lmao
    content: Vec<Ast>,
}

impl Array {
    pub fn from(content: Vec<Ast>) -> Array {
        Array { content }
    }
}

// TODO: lowkey should probably rename this to AstNode
#[derive(Debug, Clone)]
pub enum Ast {
    Literal(Literal),
    Array(Array),
    Var(String),
    Binary(Box<Ast>, TokenType, Box<Ast>),
    Func(String, Vec<String>, Vec<Ast>), // name, params, body
}

impl From<Ast> for String {
    fn from(ast: Ast) -> String {
        match ast {
            Ast::Literal(literal) => format!("{:?}", literal.content),
            Ast::Array(array) => format!("{:?}", array.content),
            Ast::Var(var) => format!("{:?}", var),
            Ast::Binary(left, op, right) => {
                format!("({:?} {:?} {:?})", left, op, right)
            }
            Ast::Func(name, params, body) => {
                format!("(fn {:?} {:?} {:?})", name, params, body)
            }
        }
    }
}
