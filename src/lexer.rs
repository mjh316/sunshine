use std::collections::HashSet;

use serde::{ser::SerializeStruct, Deserialize, Serialize};

fn KEYWORDS() -> HashSet<&'static str> {
    HashSet::from([
        "let", "=", "brush", "prep", "has", "func", "needs", "finished", "loop", "through",
        "while", "if", "elif", "else",
    ])
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Period,
    Comma,
    Colon,
    Keyword,
    Identifier,
    String,
    Number,
    Or,
    Not,
    And,
    Equiv,
    NotEquiv,
    Gt,
    Gte,
    Lt,
    Lte,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Modulo,
    EOF,
    Boolean,
}

impl Serialize for TokenType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            TokenType::LeftParen => serializer.serialize_str("("),
            TokenType::RightParen => serializer.serialize_str(")"),
            TokenType::LeftBrace => serializer.serialize_str("{"),
            TokenType::RightBrace => serializer.serialize_str("}"),
            TokenType::LeftBracket => serializer.serialize_str("["),
            TokenType::RightBracket => serializer.serialize_str("]"),
            TokenType::Period => serializer.serialize_str("."),
            TokenType::Comma => serializer.serialize_str(","),
            TokenType::Colon => serializer.serialize_str(":"),
            TokenType::Keyword => serializer.serialize_str("Keyword"),
            TokenType::Identifier => serializer.serialize_str("Identifier"),
            TokenType::String => serializer.serialize_str("String"),
            TokenType::Number => serializer.serialize_str("Number"),
            TokenType::Or => serializer.serialize_str("||"),
            TokenType::Not => serializer.serialize_str("!"),
            TokenType::And => serializer.serialize_str("&&"),
            TokenType::Equiv => serializer.serialize_str("=="),
            TokenType::NotEquiv => serializer.serialize_str("!="),
            TokenType::Gt => serializer.serialize_str(">"),
            TokenType::Gte => serializer.serialize_str(">="),
            TokenType::Lt => serializer.serialize_str("<"),
            TokenType::Lte => serializer.serialize_str("<="),
            TokenType::Plus => serializer.serialize_str("+"),
            TokenType::Minus => serializer.serialize_str("-"),
            TokenType::Modulo => serializer.serialize_str("%"),
            TokenType::Asterisk => serializer.serialize_str("*"),
            TokenType::Slash => serializer.serialize_str("/"),
            TokenType::EOF => serializer.serialize_str("EOF"),
            TokenType::Boolean => serializer.serialize_str("Boolean"),
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", format!("{:?}", self).to_lowercase())
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token {{ type: {:?},  value: {},  content: {:?}, line: {}, column: {} }}",
            self._type, self.value, self.content, self.line, self.column
        )
    }
}

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut state = serializer.serialize_struct("Token", 5)?;
        state.serialize_field("type", &self._type)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("line", &self.line)?;
        state.serialize_field("column", &self.column)?;
        state.end()
    }
}

impl TokenType {
    pub fn isOperator(&self) -> bool {
        matches!(
            self,
            TokenType::Or
                | TokenType::And
                | TokenType::Equiv
                | TokenType::NotEquiv
                | TokenType::Gt
                | TokenType::Gte
                | TokenType::Lt
                | TokenType::Lte
                | TokenType::Plus
                | TokenType::Minus
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Modulo
        )
    }

    // order of operations
    pub fn precedence(&self) -> i32 {
        match self {
            TokenType::Lt
            | TokenType::Lte
            | TokenType::Gt
            | TokenType::Gte
            | TokenType::Equiv
            | TokenType::NotEquiv
            | TokenType::And
            | TokenType::Or => 0,
            TokenType::Plus | TokenType::Minus => 1,
            TokenType::Asterisk | TokenType::Slash | TokenType::Modulo => 2,
            _ => -1,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum TokenContentType {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl Serialize for TokenContentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            TokenContentType::String(s) => serializer.serialize_str(s),
            TokenContentType::Number(n) => serializer.serialize_f64(*n),
            TokenContentType::Boolean(b) => serializer.serialize_bool(*b),
        }
    }
}

impl Into<TokenContentType> for String {
    fn into(self) -> TokenContentType {
        TokenContentType::String(self)
    }
}

impl Into<TokenContentType> for f64 {
    fn into(self) -> TokenContentType {
        TokenContentType::Number(self)
    }
}

impl Into<TokenContentType> for bool {
    fn into(self) -> TokenContentType {
        TokenContentType::Boolean(self)
    }
}

impl Into<TokenContentType> for i64 {
    fn into(self) -> TokenContentType {
        TokenContentType::Number(self as f64)
    }
}

impl From<Token> for String {
    fn from(token: Token) -> String {
        format!(
            "Token {{ type: {:?},  value: {},  content: {:?}, line: {}, column: {} }}",
            token._type, token.value, token.content, token.line, token.column
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    pub _type: TokenType,
    pub value: String,
    pub content: TokenContentType,
    line: usize,
    column: usize,
}

impl Token {
    pub fn new(
        _type: TokenType,
        value: String,
        content: TokenContentType,
        line: usize,
        column: usize,
    ) -> Token {
        Token {
            _type,
            value,
            content,
            line,
            column,
        }
    }
}

pub struct Lexer {
    program: String,
    pub tokens: Vec<Token>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(program: String) -> Lexer {
        Lexer {
            program,
            tokens: Vec::new(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn isAtEnd(&self) -> bool {
        self.current >= self.program.len()
    }

    pub fn peek(&self) -> char {
        if self.isAtEnd() {
            return '\0';
        }
        self.program.chars().nth(self.current).unwrap()
    }

    pub fn advance(&mut self) -> char {
        if self.isAtEnd() {
            return '\0';
        }

        self.column += 1;
        let ret = self.program.chars().nth(self.current).unwrap();
        self.current += 1;

        ret
    }

    pub fn match_char(&mut self, expected: char) -> Option<char> {
        if self.peek() == expected {
            return Some(self.advance());
        }
        None
    }

    fn isChar(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /**
     * Might have to return the token that gets pushed
     */
    pub fn scan_token(&mut self) {
        let char = self.advance();

        match char {
            '(' => self.tokens.push(Token::new(
                TokenType::LeftParen,
                String::from("("),
                String::from("(").into(),
                self.line,
                self.column,
            )),
            ')' => self.tokens.push(Token::new(
                TokenType::RightParen,
                String::from(")"),
                String::from(")").into(),
                self.line,
                self.column,
            )),
            '{' => self.tokens.push(Token::new(
                TokenType::LeftBrace,
                String::from("{"),
                String::from("{").into(),
                self.line,
                self.column,
            )),
            '}' => self.tokens.push(Token::new(
                TokenType::RightBrace,
                String::from("}"),
                String::from("}").into(),
                self.line,
                self.column,
            )),
            '[' => self.tokens.push(Token::new(
                TokenType::LeftBracket,
                String::from("["),
                String::from("[").into(),
                self.line,
                self.column,
            )),
            ']' => self.tokens.push(Token::new(
                TokenType::RightBracket,
                String::from("]"),
                String::from("]").into(),
                self.line,
                self.column,
            )),
            '.' => self.tokens.push(Token::new(
                TokenType::Period,
                String::from("."),
                String::from(".").into(),
                self.line,
                self.column,
            )),
            ',' => self.tokens.push(Token::new(
                TokenType::Comma,
                String::from(","),
                String::from(",").into(),
                self.line,
                self.column,
            )),
            ':' => self.tokens.push(Token::new(
                TokenType::Colon,
                String::from(":"),
                String::from(":").into(),
                self.line,
                self.column,
            )),
            '+' => self.tokens.push(Token::new(
                TokenType::Plus,
                String::from("+"),
                String::from("+").into(),
                self.line,
                self.column,
            )),
            '-' => self.tokens.push(Token::new(
                TokenType::Minus,
                String::from("-"),
                String::from("-").into(),
                self.line,
                self.column,
            )),
            '*' => self.tokens.push(Token::new(
                TokenType::Asterisk,
                String::from("*"),
                String::from("*").into(),
                self.line,
                self.column,
            )),
            '/' => self.tokens.push(Token::new(
                TokenType::Slash,
                String::from("/"),
                String::from("/").into(),
                self.line,
                self.column,
            )),
            '\'' | '"' => {
                let mut string = String::new();
                while self.peek() != char {
                    string.push(self.advance());
                    if self.isAtEnd() {
                        panic!("Unterminated string");
                    }
                }
                self.advance();
                self.tokens.push(Token::new(
                    TokenType::String,
                    string.clone(),
                    string.into(),
                    self.line,
                    self.column,
                ));
            }
            '|' => {
                if self.match_char('|').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::Or,
                        String::from("||"),
                        String::from("||").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '>' => {
                if self.match_char('=').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::Gte,
                        String::from(">="),
                        String::from(">=").into(),
                        self.line,
                        self.column,
                    ));
                } else {
                    self.tokens.push(Token::new(
                        TokenType::Gt,
                        String::from(">"),
                        String::from(">").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '<' => {
                if self.match_char('=').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::Lte,
                        String::from("<="),
                        String::from("<=").into(),
                        self.line,
                        self.column,
                    ));
                } else {
                    self.tokens.push(Token::new(
                        TokenType::Lt,
                        String::from("<"),
                        String::from("<").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '=' => {
                if self.match_char('=').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::Equiv,
                        String::from("=="),
                        String::from("==").into(),
                        self.line,
                        self.column,
                    ));
                } else {
                    self.tokens.push(Token::new(
                        TokenType::Keyword,
                        String::from("="),
                        String::from("=").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '&' => {
                if self.match_char('&').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::And,
                        String::from("&&"),
                        String::from("&&").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '!' => {
                if self.match_char('=').is_some() {
                    self.tokens.push(Token::new(
                        TokenType::NotEquiv,
                        String::from("!="),
                        String::from("!=").into(),
                        self.line,
                        self.column,
                    ));
                } else {
                    self.tokens.push(Token::new(
                        TokenType::Not,
                        String::from("!"),
                        String::from("!").into(),
                        self.line,
                        self.column,
                    ));
                }
            }
            '~' => {
                // comments!
                while self.peek() != '\n' && self.peek() != '\0' {
                    self.advance();
                }
                return;
            }
            ' ' | '\r' => {
                return;
            }
            '\n' => {
                self.line += 1;
                self.column = 0;
                return;
            }
            _ => {
                if char.is_numeric() {
                    let mut number = String::from("");
                    number.push(char);
                    let mut char = self.peek();
                    // parse decimals, too
                    while char.is_numeric() || (char == '.' && !number.contains('.')) {
                        number.push(char);
                        self.advance();
                        char = self.peek();
                    }
                    self.tokens.push(Token::new(
                        TokenType::Number,
                        number.clone(),
                        number.parse::<f64>().unwrap().into(),
                        self.line,
                        self.column,
                    ));
                } else if self.isChar(char) {
                    let mut identifier = String::from("");
                    identifier.push(char);
                    let mut char = self.peek();
                    while char.is_alphanumeric() {
                        identifier.push(char);
                        self.advance();
                        char = self.peek();
                    }

                    let _type = if KEYWORDS().contains(&identifier.as_str()) {
                        TokenType::Keyword
                    } else if identifier == "true" || identifier == "false" {
                        TokenType::Boolean
                    } else {
                        TokenType::Identifier
                    };

                    // println!("Identifier: {}", identifier);
                    self.tokens.push(Token::new(
                        _type,
                        identifier.clone(),
                        if identifier == "true" {
                            true.into()
                        } else if identifier == "false" {
                            false.into()
                        } else {
                            identifier.into()
                        },
                        self.line,
                        self.column,
                    ));
                } else {
                    panic!("Unexpected character: {}", char);
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.isAtEnd() {
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            String::from(""),
            String::from("").into(),
            self.line,
            self.column,
        ));

        self.tokens.clone()
    }
}
