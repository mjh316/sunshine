use std::collections::HashSet;

fn KEYWORDS() -> HashSet<&'static str> {
    HashSet::from([
        "prepare", "as", "brush", "prep", "has", "sketch", "needs", "finished", "loop", "through",
        "while", "if", "elif", "else",
    ])
}

#[derive(Debug, Clone, Copy)]
enum TokenType {
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
    EOF,
}

#[derive(Debug, Clone)]
enum TokenContentType {
    String(String),
    Number(f64),
    Boolean(bool),
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

impl From<Token> for String {
    fn from(token: Token) -> String {
        format!(
            "Token {{ type: {:?}, value: {}, content: {:?}, line: {}, column: {} }}",
            token._type, token.value, token.content, token.line, token.column
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    _type: TokenType,
    value: String,
    content: TokenContentType,
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
                let mut value = String::from("");
                let mut content = String::from("");
                let mut char = self.advance();
                while char != '\'' && char != '"' {
                    value.push(char);
                    content.push(char);
                    char = self.advance();

                    if char == '\0' {
                        panic!("Unterminated string literal");
                    }
                }
                self.advance(); // remove if string parsing sketch
                self.tokens.push(Token::new(
                    TokenType::String,
                    value,
                    content.into(),
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
                    } else {
                        TokenType::Identifier
                    };
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
