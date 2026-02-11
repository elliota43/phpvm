use crate::token::{Token, Spanned};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos].token;
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance().clone();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?} at token {}", expected, tok, self.pos))
        }
    }

    fn at(&self, token: &Token) -> bool {
        self.peek() == token
    }

    // -- Entry point ------------------------------------

    pub fn parse(&mut self) -> Result<Block, String> {
        // skip <?php
        self.expect(&Token::OpenTag)?;
        let mut stmts = Vec::new();
        while !self.at(&Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // -- Statements -------------------------------------

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek().clone() {
            Token::Echo => self.parse_echo(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Function => self.parse_function_def(),
            Token::Return => self.parse_return(),
            _ => {
                let expr = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_echo(&mut self) -> Result<Stmt, String> {
        self.advance();
        let mut exprs = vec![self.parse_expr()?];
        while self.at(&Token::Comma) {
            self.advance();
            exprs.push(self.parse_expr()?);
        }
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Echo(exprs))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'if'
        self.expect(&Token::OpenParen)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::CloseParen)?;
        let then_block = self.parse_block()?;

        let mut elseif_blocks = Vec::new();
        let mut else_block = None;

        loop {
            if self.at(&Token::Elseif) {
                self.advance();
                self.expect(&Token::OpenParen)?;
                let cond = self.parse_expr()?;
                self.expect(&Token::CloseParen)?;
                let block = self.parse_block()?;
                elseif_blocks.push((cond, block));
            } else if self.at(&Token::Else) {
                self.advance();
                else_block = Some(self.parse_block()?);
                break;
            } else {
                break;
            }
        }

        Ok(Stmt::If { condition, then_block, elseif_blocks, else_block })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'while'
        self.expect(&Token::OpenParen)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::CloseParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'for'
        self.expect(&Token::OpenParen)?;

        let init = if self.at(&Token::Semicolon) { None } else { Some(self.parse_expr()?) };
        self.expect(&Token::Semicolon)?;

        let condition = if self.at(&Token::Semicolon) { None } else { Some(self.parse_expr()?) };
        self.expect(&Token::Semicolon)?;

        let update = if self.at(&Token::CloseParen) { None } else { Some(self.parse_expr()?) };
        self.expect(&Token::CloseParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::For { init, condition, update, body })
    }

    fn parse_function_def(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'function'
        let name = match self.advance().clone() {
            Token::Identifier(n) => n,
            t => return Err(format!("Expected function name, got {:?}", t)),
        };
        self.expect(&Token::OpenParen)?;

        let mut params = Vec::new();
        if !self.at(&Token::CloseParen) {
            loop {
                match self.advance().clone() {
                    Token::Variable(p) => params.push(p),
                    t => return Err(format!("Expected parameter name, got {:?}", t))
                }
                if self.at(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::CloseParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::FunctionDef { name, params, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'
        if self.at(&Token::Semicolon) {
            self.advance();
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(&Token::OpenBrace)?;
        let mut stmts = Vec::new();
        while !self.at(&Token::CloseBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::CloseBrace)?;
        Ok(stmts)
    }

    // -- Expressions (precedence climbing) ---------------------

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let expr = self.parse_or()?;

        if self.at(&Token::Assign) {
            self.advance();
            let value = self.parse_assignment()?; // right-associative
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    variable: name,
                    value: Box::new(value),
                }),
                Expr::ArrayAccess { array, index } => {
                    Ok(Expr::Assign {
                        variable: format!("__array_set"),
                        value: Box::new(value),
                    })
                }
                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.at(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.at(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Token::Equal => BinOp::Equal,
                Token::Identical => BinOp::Identical,
                Token::NotEqual => BinOp::NotEqual,
                Token::NotIdentical => BinOp::NotIdentical,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Token::Less => BinOp::Less,
                Token::LessEqual => BinOp::LessEqual,
                Token::Greater => BinOp::Greater,
                Token::GreaterEqual => BinOp::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        while self.at(&Token::Dot) {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Concat,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Negate, expr: Box::new(expr) })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.at(&Token::OpenBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(&Token::CloseBracket)?;
                expr = Expr::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Integer(n) => { self.advance(); Ok(Expr::Integer(n)) }
            Token::Float(n) => { self.advance(); Ok(Expr::Float(n)) }
            Token::StringLiteral(s) => { self.advance(); Ok(Expr::String(s)) }
            Token::True => { self.advance(); Ok(Expr::Bool(true)) }
            Token::False => { self.advance(); Ok(Expr::Bool(false)) }
            Token::Null => { self.advance(); Ok(Expr::Null) }

            Token::Variable(name) => {
                self.advance();
                Ok(Expr::Variable(name))
            }

            Token::Identifier(name) => {
                self.advance();
                // function call
                if self.at(&Token::OpenParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.at(&Token::CloseParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.at(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::CloseParen)?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    // bare identifier — treat as string constant or error
                    Err(format!("Unexpected identifier '{}' (not a function call)", name))
                }
            }

            Token::OpenParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::CloseParen)?;
                Ok(expr)
            }

            Token::OpenBracket => {
                self.advance();
                let mut entries = Vec::new();
                if !self.at(&Token::CloseBracket) {
                    loop {
                        let first = self.parse_expr()?;
                        if self.at(&Token::Arrow) {
                            self.advance();
                            let value = self.parse_expr()?;
                            entries.push(ArrayEntry { key: Some(first), value });
                        } else {
                            entries.push(ArrayEntry { key: None, value: first });
                        }
                        if self.at(&Token::Comma) {
                            self.advance();
                            // allow trailing comma
                            if self.at(&Token::CloseBracket) { break; }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::CloseBracket)?;
                Ok(Expr::ArrayLiteral(entries))
            }

            t => Err(format!("Unexpected token {:?}", t)),
        }
    }

}