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
    /**
     * name, params, body
     */
    Func(String, Vec<String>, Vec<Ast>),
    Return(Box<Ast>),
    /**
     * id, range, body
     */
    For(String, Vec<Ast>, Vec<Ast>),
    /**
     * condition, body
     */
    While(Box<Ast>, Vec<Ast>),
    /**
     * condition, if body, else body
     */
    Conditional(Box<Ast>, Vec<Ast>, Vec<Ast>),
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
            Ast::Return(expr) => format!("(return {:?})", expr),
            Ast::For(id, range, body) => {
                format!("(for {:?} {:?} {:?})", id, range, body)
            }
            Ast::While(condition, body) => {
                format!("(while {:?} {:?})", condition, body)
            }
            Ast::Conditional(condition, if_body, else_body) => {
                format!("(if {:?} {:?} {:?})", condition, if_body, else_body)
            }
        }
    }
}
