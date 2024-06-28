use std::{collections::HashMap, ops::Not};

use crate::lexer::{TokenContentType, TokenType};

#[derive(Debug, Clone)]
pub struct Literal {
    pub content: TokenContentType,
}

impl Literal {
    pub fn from(content: TokenContentType) -> Literal {
        Literal { content }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    // TODO: fix this vec type lmao
    pub content: Vec<Ast>,
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
    /**
     * name, value
     */
    Var(String, Option<Box<Ast>>),
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
    Set(String, String, Box<Ast>),
    Struct(String, Vec<String>),
    Instance(String, HashMap<String, Ast>),
    Call(Box<Ast>, Vec<Ast>),
    Get(Box<Ast>, Box<Ast>, bool),
    PointGet(Box<Ast>, String),
    Unary(TokenType, Box<Ast>),
}

impl Clone for Ast {
    fn clone(&self) -> Self {
        match self {
            Ast::Literal(lit) => Ast::Literal(lit.clone()),
            Ast::Array(arr) => Ast::Array(arr.clone()),
            Ast::Var(name, value) => Ast::Var(name.clone(), value.clone()),
            Ast::Binary(left, op, right) => Ast::Binary(left.clone(), op.clone(), right.clone()),
            Ast::Func(name, params, body) => Ast::Func(name.clone(), params.clone(), body.clone()),
            Ast::Return(value) => Ast::Return(value.clone()),
            Ast::For(id, range, body) => Ast::For(id.clone(), range.clone(), body.clone()),
            Ast::While(cond, body) => Ast::While(cond.clone(), body.clone()),
            Ast::Conditional(cond, if_body, else_body) => {
                Ast::Conditional(cond.clone(), if_body.clone(), else_body.clone())
            }
            Ast::Set(obj, field, value) => Ast::Set(obj.clone(), field.clone(), value.clone()),
            Ast::Struct(name, fields) => Ast::Struct(name.clone(), fields.clone()),
            Ast::Instance(name, fields) => Ast::Instance(name.clone(), fields.clone()),
            Ast::Call(callee, args) => Ast::Call(callee.clone(), args.clone()),
            Ast::Get(obj, prop, is_bracket) => Ast::Get(obj.clone(), prop.clone(), *is_bracket),
            Ast::PointGet(obj, field) => Ast::PointGet(obj.clone(), field.clone()),
            Ast::Unary(op, expr) => Ast::Unary(op.clone(), expr.clone()),
            Ast::Invoke(func) => Ast::Invoke(Box::new(func.clone())),
        }
    }
}

impl From<Ast> for String {
    fn from(ast: Ast) -> String {
        match ast {
            Ast::Literal(literal) => format!("{:?}", literal.content),
            Ast::Array(array) => format!("{:?}", array.content),
            Ast::Var(name, value) => {
                if let Some(value) = value {
                    format!("(var {:?} = {:?})", name, value)
                } else {
                    format!("(var {:?}) = None", name)
                }
            }
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
            Ast::Set(caller, property, value) => {
                format!("(set {:?} {:?} {:?})", caller, property, value)
            }
            Ast::Struct(name, fields) => {
                format!("(struct {:?} {:?})", name, fields)
            }
            Ast::Instance(name, fields) => {
                format!("(instance {:?} {:?})", name, fields)
            }
            Ast::Call(caller, args) => {
                format!("(call {:?} {:?})", caller, args)
            }
            Ast::Get(caller, property, is_method) => {
                format!("(get {:?} {:?} {:?})", caller, property, is_method)
            }
            Ast::PointGet(caller, property) => {
                format!("(point-get {:?} {:?})", caller, property)
            }
            Ast::Unary(op, expr) => {
                format!("({:?} {:?})", op, expr)
            }
        }
    }
}

impl Not for Ast {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Boolean(b) => Ast::Literal(Literal {
                    content: TokenContentType::Boolean(!b),
                }),
                _ => panic!("Expected boolean literal but got {:?}", literal.content),
            },
            _ => panic!("Expected boolean literal but got {:?}", self),
        }
    }
}
