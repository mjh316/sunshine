use core::fmt;
use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Not, Rem, Sub},
    os::macos::raw::stat,
};

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
    None,
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
                let mut state = serializer.serialize_map(Some(2))?;
                let r#type = "Literal";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("value", &literal.content)?;
                return state.end();
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
                let mut state = serializer.serialize_map(Some(4))?;
                let r#type = "Binary";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("left", &left)?;
                state.serialize_entry("operator", &op)?;
                state.serialize_entry("right", &right)?;
                return state.end();
                // let mut state = serializer.serialize_seq(Some(3))?;
                // state.serialize_element(&left)?;
                // state.serialize_element(&op)?;
                // state.serialize_element(&right)?;
                // return state.end();
            }
            Ast::Func(name, params, body) => {
                let mut state = serializer.serialize_map(Some(4))?;
                let r#type = "Func";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("name", &name)?;
                state.serialize_entry("params", &params)?;
                state.serialize_entry("body", &body)?;
                return state.end();
            }
            Ast::Return(expr) => {
                let mut state = serializer.serialize_map(Some(2))?;
                let r#type = "Return";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("value", &expr)?;
                return state.end();
            }
            Ast::For(id, range, body) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "For";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("id", &id)?;
                state.serialize_entry("range", &range)?;
                state.serialize_entry("body", &body)?;
                return state.end();
            }
            Ast::While(condition, body) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "While";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("condition", &condition)?;
                state.serialize_entry("body", &body)?;
                return state.end();
            }
            Ast::Conditional(condition, r#if, r#else) => {
                let mut state = serializer.serialize_map(Some(4))?;
                let r#type = "Conditional";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("condition", &condition)?;
                state.serialize_entry("body", &r#if)?;
                state.serialize_entry("otherwise", &r#else)?;
                return state.end();
            }
            Ast::Set(caller, property, value) => {
                let mut state = serializer.serialize_map(Some(4))?;
                let r#type = "Set";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("caller", &caller)?;
                state.serialize_entry("property", &property)?;
                state.serialize_entry("value", &value)?;
                return state.end();
            }
            Ast::Struct(name, members) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "Struct";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("name", &name)?;
                state.serialize_entry("members", &members)?;
                return state.end();
            }
            Ast::Instance(name, members) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "Instance";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("name", &name)?;
                state.serialize_entry("members", &members)?;
                return state.end();
            }
            Ast::Call(caller, args) => {
                /*
                 * TODO: make sure that this works when caller is an AST and not a primitive string
                 */
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "Call";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("caller", &caller)?;
                state.serialize_entry("args", &args)?;
                return state.end();
            }
            Ast::Get(caller, property, is_expr) => {
                let mut state = serializer.serialize_map(Some(4))?;
                let r#type = "Get";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("caller", &caller)?;
                state.serialize_entry("property", &property)?;
                state.serialize_entry("isExpr", &is_expr)?;
                return state.end();
            }
            Ast::Unary(operator, apply) => {
                let mut state = serializer.serialize_map(Some(3))?;
                let r#type = "Unary";
                state.serialize_entry("type", &r#type)?;
                state.serialize_entry("operator", &operator)?;
                state.serialize_entry("apply", &apply)?;
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
            Ast::None => "None".to_string(),
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

impl Add for Ast {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        // support strings and numbers
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => Ast::Literal(Literal {
                            content: TokenContentType::Number(n + other_n),
                        }),
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                TokenContentType::String(s) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::String(other_s) => Ast::Literal(Literal {
                            content: TokenContentType::String(format!("{}{}", s, other_s)),
                        }),
                        _ => panic!(
                            "Expected string literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected string literal but got {:?}", other),
                },
                _ => panic!(
                    "Expected number or string literal but got {:?}",
                    literal.content
                ),
            },
            _ => panic!("Expected number or string literal but got {:?}", self),
        }
    }
}

impl Sub for Ast {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        // support strings and numbers
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => Ast::Literal(Literal {
                            content: TokenContentType::Number(n - other_n),
                        }),
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                _ => panic!(
                    "Expected number or string literal but got {:?}",
                    literal.content
                ),
            },
            _ => panic!("Expected number or string literal but got {:?}", self),
        }
    }
}

impl Mul for Ast {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        // support strings and numbers
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => Ast::Literal(Literal {
                            content: TokenContentType::Number(n * other_n),
                        }),
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                TokenContentType::String(s) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => {
                            if other_n.fract() != 0.0 {
                                panic!("Cannot multiply string by non-integer number")
                            }
                            if other_n < 0.0 {
                                panic!("Cannot multiply string by negative number")
                            }
                            let mut result = String::new();
                            for _ in 0..(other_n as u64) {
                                result.push_str(&s);
                            }
                            Ast::Literal(Literal {
                                content: TokenContentType::String(result),
                            })
                        }
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                _ => panic!(
                    "Expected number or string literal but got {:?}",
                    literal.content
                ),
            },
            _ => panic!("Expected number or string literal but got {:?}", self),
        }
    }
}

impl Div for Ast {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        // support strings and numbers
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => {
                            if other_n == 0.0 {
                                panic!("Cannot divide by zero")
                            }
                            Ast::Literal(Literal {
                                content: TokenContentType::Number(n / other_n),
                            })
                        }
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                _ => panic!(
                    "Expected number or string literal but got {:?}",
                    literal.content
                ),
            },
            _ => panic!("Expected number or string literal but got {:?}", self),
        }
    }
}

impl Rem for Ast {
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        // support strings and numbers
        match self {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => {
                            if other_n == 0.0 {
                                panic!("Cannot get remainder by zero")
                            }
                            Ast::Literal(Literal {
                                content: TokenContentType::Number(n % other_n),
                            })
                        }
                        _ => panic!(
                            "Expected number literal but got {:?}",
                            other_literal.content
                        ),
                    },
                    _ => panic!("Expected number literal but got {:?}", other),
                },
                _ => panic!(
                    "Expected number or string literal but got {:?}",
                    literal.content
                ),
            },
            _ => panic!("Expected number or string literal but got {:?}", self),
        }
    }
}

impl PartialEq for Ast {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Ast::Literal(literal) => match literal.content.clone() {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => n == other_n,
                        _ => false,
                    },
                    _ => false,
                },
                TokenContentType::String(s) => match other {
                    Ast::Literal(other_literal) => match other_literal.content.clone() {
                        TokenContentType::String(other_s) => s == other_s,
                        _ => false,
                    },
                    _ => false,
                },
                TokenContentType::Boolean(b) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Boolean(other_b) => b == other_b,
                        _ => false,
                    },
                    _ => false,
                },
            },
            Ast::Var(_name, value) => match other {
                Ast::Var(_other_name, other_value) => value == other_value,
                _ => false,
            },
            Ast::Array(array) => match other {
                Ast::Array(other_array) => &array.content == &other_array.content,
                _ => false,
            },
            _ => false,
        }
    }
}

impl PartialOrd for Ast {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Ast::Literal(literal) => match literal.content.clone() {
                TokenContentType::Number(n) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Number(other_n) => n.partial_cmp(&other_n),
                        _ => None,
                    },
                    _ => None,
                },
                TokenContentType::String(s) => match other {
                    Ast::Literal(other_literal) => match other_literal.content.clone() {
                        TokenContentType::String(other_s) => s.partial_cmp(&other_s),
                        _ => None,
                    },
                    _ => None,
                },
                TokenContentType::Boolean(b) => match other {
                    Ast::Literal(other_literal) => match other_literal.content {
                        TokenContentType::Boolean(other_b) => b.partial_cmp(&other_b),
                        _ => None,
                    },
                    _ => None,
                },
            },
            Ast::Var(_name, value) => match other {
                Ast::Var(_other_name, other_value) => value.partial_cmp(&other_value),
                _ => None,
            },
            Ast::Array(array) => match other {
                Ast::Array(other_array) => (&array.content).partial_cmp(&other_array.content),
                _ => None,
            },
            _ => None,
        }
    }
}
