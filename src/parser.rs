use crate::ast::Array;
use crate::ast::Ast;
use crate::ast::Literal;
use crate::lexer::Token;
use crate::lexer::TokenType;
use std::borrow::Cow;

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

    fn identifierList(&mut self) -> Vec<String> {
        let mut identifiers = vec![];
        identifiers.push(self.eat(TokenType::Identifier).value);
        loop {
            let next = self.peek();
            match next {
                Some(token) => match token._type {
                    TokenType::Comma => {
                        self.eat(TokenType::Comma);
                        identifiers.push(self.eat(TokenType::Identifier).value);
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
        identifiers
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
            TokenType::LeftParen => {
                let expr = self.expr();
                self.eat(TokenType::RightParen);
                return expr;
            }
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

    fn peekKeyword(&self, keyword: &'static str) -> Option<Token> {
        let nextType = self.peekType().unwrap();
        match nextType {
            TokenType::Keyword => {
                if self.peek().unwrap().value == keyword {
                    return Some(self.peek().unwrap());
                }
            }
            _ => {}
        }
        None
    }

    fn peekKeywordOwned(&self, keyword: String) -> Option<Token> {
        let nextType = self.peekType().unwrap();
        match nextType {
            TokenType::Keyword => {
                if self.peek().unwrap().value == keyword {
                    return Some(self.peek().unwrap());
                }
            }
            _ => {}
        }
        None
    }

    fn eatKeyword(&mut self, keyword: &'static str) -> Token {
        let next = self.peekKeyword(keyword);
        match next {
            Some(token) => {
                if token.value != keyword {
                    panic!("Expected keyword {:?}, got {:?}", keyword, token.value);
                }
                self.eat(TokenType::Keyword)
            }
            None => {
                panic!("Expected keyword {:?}, got None", keyword);
            }
        }
    }

    fn eatKeywordOwned(&mut self, keyword: String) -> Token {
        let next = self.peekKeywordOwned(keyword.clone());
        match next {
            Some(token) => {
                if token.value != keyword {
                    panic!("Expected keyword {:?}, got {:?}", keyword, token.value);
                }
                self.eat(TokenType::Keyword)
            }
            None => {
                panic!("Expected keyword {:?}, got None", keyword);
            }
        }
    }

    fn funcStmt(&mut self) -> Ast {
        self.eatKeyword("sketch");
        let name = self.eat(TokenType::Identifier).value;

        let mut params = vec![];
        if self
            .peekKeyword("needs")
            .is_some_and(|x| x.value == "needs")
        {
            self.eatKeyword("needs");
            self.eat(TokenType::LeftParen);
            params = self.identifierList();
            self.eat(TokenType::RightParen);
        }

        self.eat(TokenType::LeftBrace);
        let mut body = vec![];
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            body.push(self.stmt());
        }
        self.eat(TokenType::RightBrace);

        Ast::Func(name, params, body)
    }

    fn returnStmt(&mut self) -> Ast {
        self.eatKeyword("finished");
        let expr = self.expr();
        Ast::Return(Box::new(expr))
    }

    fn forStmt(&mut self) -> Ast {
        self.eatKeyword("loop");
        let id = self.eat(TokenType::Identifier).value;
        self.eatKeyword("through");

        self.eat(TokenType::LeftParen);
        let range = self.exprList();
        if range.len() != 2 {
            panic!("Expected range to have 2 elements, got {:?}", range.len());
        }
        self.eat(TokenType::RightParen);

        self.eat(TokenType::LeftBrace);
        let mut body = vec![];
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            body.push(self.stmt());
        }
        self.eat(TokenType::RightBrace);

        Ast::For(id, range, body)
    }

    fn whileStmt(&mut self) -> Ast {
        self.eatKeyword("while");

        self.eat(TokenType::LeftParen);
        let condition = self.expr();
        self.eat(TokenType::RightParen);

        self.eat(TokenType::LeftBrace);
        let mut body = vec![];
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            body.push(self.stmt());
        }
        self.eat(TokenType::RightBrace);

        return Ast::While(Box::new(condition), body);
    }

    fn conditionalStmt<'a>(&mut self, keyword: String) -> Ast {
        self.eatKeywordOwned(keyword.clone());

        let mut condition = Ast::Literal(Literal::from(true.into()));
        if keyword != "else" {
            self.eat(TokenType::LeftParen);
            condition = self.expr();
            self.eat(TokenType::RightParen);
        }

        self.eat(TokenType::LeftBrace);
        let mut body = vec![];
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            body.push(self.stmt());
        }
        self.eat(TokenType::RightBrace);

        let mut otherwise = vec![];
        while self.peekKeyword("else").is_some_and(|x| x.value == "else")
            || self.peekKeyword("elif").is_some_and(|x| x.value == "elif")
        {
            let next = self.peek().unwrap();
            otherwise.push(self.conditionalStmt(next.value.clone()));
        }

        Ast::Conditional(Box::new(condition), body, otherwise)
    }

    pub fn stmt(&mut self) -> Ast {
        let next = self.peek();
        match next {
            Some(token) => match token._type {
                TokenType::Keyword => match token.value.as_str() {
                    "sketch" => {
                        return self.funcStmt();
                    }
                    "finished" => {
                        return self.returnStmt();
                    }
                    "loop" => {
                        return self.forStmt();
                    }
                    "while" => {
                        return self.whileStmt();
                    }
                    "if" => {
                        return self.conditionalStmt("if".to_owned());
                    }
                    _ => {
                        panic!("Unexpected keyword: {:?}", token.value);
                    }
                },
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
