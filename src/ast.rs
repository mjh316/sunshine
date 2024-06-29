use std::{collections::HashMap, ops::Not};

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

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
    // result of setting up a closure
}

impl Serialize for Ast {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.clone() {
            Ast::Array(array) => {
                let mut stateMap = serializer.serialize_map(Some(2))?;
                let r#type = "Array";
                stateMap.serialize_entry("type", &r#type)?;
                stateMap.serialize_entry("value", &array.content)?;
                return stateMap.end();
            }
            Ast::Literal(literal) => {
                // ?
                return literal.content.serialize(serializer);
            }
            Ast::Var(name, value) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "Var";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("name", &name)?;
                state.serialize_entry("value", &value)?;
                return state.end();
                // state.serialize_element(&r#type)?;
                // state.serialize_element(&name)?;
                // let value: Ast = *value.unwrap();
                // state.serialize_element(&value)?;
                // return state.end();
            }
            Ast::Binary(left, op, right) => {
                let mut state = serializer.serialize_seq(Some(3))?;
                state.serialize_element(&left)?;
                state.serialize_element(&op)?;
                state.serialize_element(&right)?;
                return state.end();
            }
            Ast::Func(name, params, body) => {
                let mut state = serializer.serialize_seq(Some(3))?;
                state.serialize_element(&name)?;
                state.serialize_element(&params)?;
                state.serialize_element(&body)?;
                return state.end();
            }
            Ast::Return(expr) => {
                let mut state = serializer.serialize_seq(Some(1))?;
                state.serialize_element(&expr)?;
                return state.end();
            }
            _ => unimplemented!(),
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
