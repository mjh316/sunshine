use crate::ast::Array;
use crate::ast::Ast;
use crate::ast::Literal;
use crate::lexer::Token;
use crate::lexer::TokenType;

pub struct Parser {
    tokens: Vec<Token>,
    ast: Vec<Ast>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            ast: vec![],
            current: 0,
        }
    }

    fn peek(&self) -> Option<Token> {
        if self.current >= self.tokens.len() {
            return None;
        }
        Some(self.tokens[self.current].clone())
    }

    fn peekType(&self) -> Option<TokenType> {
        if self.current >= self.tokens.len() {
            return None;
        }
        Some(self.tokens[self.current]._type)
    }

    pub fn parse(&mut self) -> Vec<Ast> {
        while let Some(nextType) = self.peekType() {
            // If we reach the end of the file, break
            if matches!(nextType, TokenType::EOF) {
                break;
            }

            let stmt = self.stmt();
            self.ast.push(stmt);
        }

        self.ast.clone()
    }

    pub fn eat(&mut self, tokenType: TokenType) -> Token {
        match self.peekType() {
            Some(tokenType) => {
                self.current += 1;
                return self.tokens[self.current - 1].clone();
            }
            None => {
                panic!("Expected token type {:?}, got None", tokenType);
            }
        }
    }

    pub fn simple(&mut self) -> Ast {
        let token = self.eat(self.peekType().unwrap());
        match token._type {
            TokenType::String | TokenType::Number | TokenType::Boolean => {
                return Ast::Literal(Literal::from(token.content.clone()));
            }
            TokenType::LeftBracket => {
                let mut items = Vec::new();
                let nextType = self.peekType();
                if !nextType.is_some_and(|x| matches!(x, TokenType::RightBracket)) {
                    items = self.exprList();
                }
                self.eat(TokenType::RightBracket);
                return Ast::Array(Array::from(items));
            }
            TokenType::Identifier => return Ast::Var(token.value.clone()),
            _ => {
                panic!("Unexpected token: {:?}", token);
            }
        }
    }

    pub fn expr(&mut self) -> Ast {
        let left = self.simple();
        if self.peekType().unwrap().isOperator() {
            let op = self.eat(self.peekType().unwrap())._type;
            let right = self.expr();
            match right.clone() {
                Ast::Binary(rightLeft, rightOp, rightRight) => {
                    if op.precedence() > rightOp.precedence() {
                        return Ast::Binary(
                            Box::new(Ast::Binary(Box::new(left), op, rightLeft)),
                            rightOp,
                            rightRight,
                        );
                    }
                    return Ast::Binary(Box::new(left), op, Box::new(right));
                }
                _ => {}
            }
            return Ast::Binary(Box::new(left), op, Box::new(right));
        }
        left
    }

    pub fn exprList(&mut self) -> Vec<Ast> {
        let mut exprs = vec![];
        exprs.push(self.expr());
        loop {
            let next = self.peek();
            match next {
                Some(token) => match token._type {
                    TokenType::Comma => {
                        self.eat(TokenType::Comma);
                        exprs.push(self.expr());
                    }
                    _ => {
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
        exprs
    }

    pub fn stmt(&mut self) -> Ast {
        let next = self.peek();
        match next {
            Some(token) => match token._type {
                _ => {
                    return self.expr();
                    // panic!("Unexpected token: {:?}", token);
                }
            },
            None => {
                panic!("Unexpected EOF");
            }
        }
    }
}
