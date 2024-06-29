use crate::ast::Array;
use crate::ast::Ast;
use crate::ast::Literal;
use crate::lexer::Token;
use crate::lexer::TokenType;
use std::borrow::Cow;
use std::collections::HashMap;

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

    pub fn eat(&mut self, token_type: TokenType) -> Token {
        match self.peekType() {
            Some(tokenType2) => {
                if tokenType2.to_string() != token_type.to_string() {
                    panic!(
                        "Expected token type {:?}, got {:?}",
                        token_type,
                        self.peek().unwrap()
                    );
                }
                self.current += 1;
                return self.tokens[self.current - 1].clone();
            }
            None => {
                panic!("Expected token type {:?}, got None", token_type);
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
            TokenType::Identifier => return Ast::Var(token.value.clone(), None),
            TokenType::LeftParen => {
                let expr = self.expr();
                // println!("RIGHTPAREN 1");
                self.eat(TokenType::RightParen);
                return expr;
            }
            TokenType::Keyword => match token.value.as_str() {
                "prep" => {
                    let id = self.eat(TokenType::Identifier).value.clone();

                    self.eat(TokenType::LeftParen);

                    let mut members: HashMap<String, Ast> = HashMap::new();
                    while !matches!(self.peekType().unwrap(), TokenType::RightParen) {
                        let member = self.eat(TokenType::Identifier).value.clone();
                        self.eat(TokenType::Colon);
                        members.insert(member, self.expr());
                        if matches!(self.peekType().unwrap(), TokenType::Comma) {
                            self.eat(TokenType::Comma);
                        }
                    }

                    // println!("RIGHTPAREN 2");
                    self.eat(TokenType::RightParen);

                    return Ast::Instance(id, members);
                }
                _ => {
                    panic!("Unexpected keyword: {:?}", token.value);
                }
            },
            _ => {
                panic!("Unexpected token: {:?}", token);
            }
        }
    }

    fn call(&mut self) -> Ast {
        let mut expr = self.simple();
        loop {
            match self.peekType().unwrap() {
                TokenType::LeftParen => {
                    self.eat(TokenType::LeftParen);

                    let mut args = vec![];
                    if !matches!(self.peekType().unwrap(), TokenType::RightParen) {
                        args = self.exprList();
                        // println!("args: {:?}", args);
                    } else {
                        // println!("no arguments to function call: {:?}", expr);
                    }

                    // println!("RIGHTPAREN 3");
                    // println!(
                    // "eaten right parens in call: {:?}",
                    self.eat(TokenType::RightParen);
                    // );
                    // println!("self.current in call: {:?}", self.current);
                    println!("eaten right parens in call: {:?}", expr);
                    expr = Ast::Call(Box::new(expr), args);

                    // println!("expr: {:?}", expr);
                }
                TokenType::LeftBracket => {
                    self.eat(TokenType::LeftBracket);
                    let property = self.expr();
                    self.eat(TokenType::RightBracket);
                    expr = Ast::Get(Box::new(expr), Box::new(property), true);
                }
                TokenType::Period => {
                    self.eat(TokenType::Period);
                    let property = self.eat(TokenType::Identifier).value.clone();
                    expr = Ast::PointGet(Box::new(expr), property);
                }
                _ => break,
            }
        }
        expr
    }

    fn unary(&mut self) -> Ast {
        match self.peekType().unwrap() {
            TokenType::Not => {
                let op = self.eat(self.peekType().unwrap()).value;
                Ast::Unary(TokenType::Not, Box::new(self.unary()))
            }
            _ => self.call(),
        }
    }

    pub fn expr(&mut self) -> Ast {
        let left = self.unary();
        // println!("left: {:?}", left);
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
            // println!("RIGHTPAREN 4");
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
        // println!("RIGHTPAREN 5");
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
        // println!("RIGHTPAREN 6");
        self.eat(TokenType::RightParen);

        self.eat(TokenType::LeftBrace);
        let mut body = vec![];
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            body.push(self.stmt());
        }
        self.eat(TokenType::RightBrace);

        return Ast::While(Box::new(condition), body);
    }

    // todo: you could totally avoid the nonsense for the conditionalStmt
    // if you just used separate if statements for the otherwise and elif
    fn conditionalStmt(&mut self, keyword: &'static str) -> Ast {
        self.eatKeyword(keyword);

        let mut condition = Ast::Literal(Literal::from(true.into()));
        if keyword != "else" {
            self.eat(TokenType::LeftParen);
            condition = self.expr();
            // println!("RIGHTPAREN 7");
            // println!("self.current in conditionalStmt: {:?}", self.current);
            self.eat(TokenType::RightParen);
        }

        // println!("self.current1 in conditionalStmt: {:?}", self.current);
        self.eat(TokenType::LeftBrace);
        // println!("self.current2 in conditionalStmt: {:?}", self.current);
        let mut body = vec![];
        // println!("starting conditional body: {:?}", body);
        while !matches!(self.peekType().unwrap(), TokenType::RightBrace) {
            // println!("peekType: {:?}", self.peekType().unwrap());
            // println!("self.current3 in conditionalStmt: {:?}", self.current);
            body.push(self.stmt());
            // println!("self.current4 in conditionalStmt: {:?}", self.current);
            // println!("conditional body: {:?}", body)
        }
        // println!("end of conditional body: {:?}", body);
        // println!(
        // "eaten right brace in conditionalStmt: {:?}",
        self.eat(TokenType::RightBrace);
        // );

        let mut otherwise = vec![];
        loop {
            let elseKeyword = self.peekKeyword("else");
            let elifKeyword = self.peekKeyword("elif");

            if elseKeyword.is_some() {
                otherwise.push(self.conditionalStmt("else"));
            } else if elifKeyword.is_some() {
                otherwise.push(self.conditionalStmt("elif"));
            } else {
                break;
            }
        }

        Ast::Conditional(Box::new(condition), body, otherwise)
    }

    fn assignStmt(&mut self) -> Ast {
        self.eatKeyword("prepare");
        let name = self.eat(TokenType::Identifier).value;

        if matches!(self.peekType().unwrap(), TokenType::Period) {
            self.eat(TokenType::Period);
            let property = self.eat(TokenType::Identifier).value;
            self.eatKeyword("as");
            let value = self.expr();
            return Ast::Set(name, property, Box::new(value.clone()));
        }

        self.eatKeyword("as");
        let value = self.expr();
        Ast::Var(name, Some(Box::new(value)))
    }

    fn structStmt(&mut self) -> Ast {
        self.eatKeyword("brush");
        let name = self.eat(TokenType::Identifier).value;
        self.eatKeyword("has"); // todo: remove this or change it

        self.eat(TokenType::LeftBrace);
        let members = self.identifierList();
        self.eat(TokenType::RightBrace);

        Ast::Struct(name, members)
    }

    pub fn stmt(&mut self) -> Ast {
        let next = self.peek();
        // println!("next token in stmt(): {:?}", next);
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
                        return self.conditionalStmt("if");
                    }
                    "prepare" => {
                        return self.assignStmt();
                    }
                    "brush" => {
                        return self.structStmt();
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
