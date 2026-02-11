use crate::token::{Spanned, Token};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.peek_next() == Some('/') {
                // line comment
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if c == '/' && self.peek_next() == Some('*') {
                // block comment
                self.advance(); // /
                self.advance(); // *
                loop {
                    match self.advance() {
                        Some('*') if self.peek() == Some('/') => {
                            self.advance();
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }
            } else if c == '#' {
                // # comment
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn spanned(&self, token: Token, line: usize, col: usize) -> Spanned {
        Spanned { token, line, col }
    }

    fn read_string(&mut self, quote: char) -> Token {
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some(c) if c == quote => s.push(c),
                    Some(c) => {
                        s.push('\\');
                        s.push(c);
                    }
                    None => break,
                },
                Some(c) if c == quote => break,
                Some(c) => s.push(c),
                None => break, // unterminated string, could error
            }
        }
        Token::StringLiteral(s)
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut num = String::new();
        num.push(first);
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                // check if it is ..
                if self.peek_next().map_or(false, |n| n.is_ascii_digit()) {
                    is_float = true;
                    num.push(c);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(num.parse().unwrap())
        } else {
            Token::Integer(num.parse().unwrap())
        }
    }

    fn read_identifier(&mut self, first: char) -> String {
        let mut ident = String::new();
        ident.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn keyword_or_ident(&self, ident: &str) -> Token {
        match ident {
            "echo" => Token::Echo,
            "if" => Token::If,
            "else" => Token::Else,
            "elseif" => Token::Elseif,
            "while" => Token::While,
            "for" => Token::For,
            "function" => Token::Function,
            "return" => Token::Return,
            "true" | "TRUE" => Token::True,
            "false" | "FALSE" => Token::False,
            "null" | "NULL" => Token::Null,
            _ => Token::Identifier(ident.to_string()),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Spanned>, String> {
        let mut tokens = Vec::new();

        // Expect <?php at start
        self.skip_whitespace();
        if self.source.len() >= 5 {
            let tag: String = self.source[self.pos..self.pos + 5].iter().collect();
            if tag == "<?php" {
                let line = self.line;
                let col = self.col;
                for _ in 0..5 {
                    self.advance();
                }
                tokens.push(self.spanned(Token::OpenTag, line, col));
            } else {
                return Err(format!("Expected <?php at start, got {:?}", tag));
            }
        } else {
            return Err("Expected <?php".to_string());
        }

        loop {
            self.skip_whitespace();
            let line = self.line;
            let col = self.col;

            let ch = match self.advance() {
                Some(c) => c,
                None => {
                    tokens.push(self.spanned(Token::Eof, line, col));
                    break;
                }
            };

            let token = match ch {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '%' => Token::Percent,
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,
                '{' => Token::OpenBrace,
                '}' => Token::CloseBrace,
                '[' => Token::OpenBracket,
                ']' => Token::CloseBracket,
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                '.' => Token::Dot,
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Token::Identical
                        } else {
                            Token::Equal
                        }
                    } else if self.peek() == Some('>') {
                        self.advance();
                        Token::Arrow
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Token::NotIdentical
                        } else {
                            Token::NotEqual
                        }
                    } else {
                        Token::Not
                    }
                }

                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }

                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }

                '&' if self.peek() == Some('&') => {
                    self.advance();
                    Token::And
                }

                '|' if self.peek() == Some('|') => {
                    self.advance();
                    Token::Or
                }

                '$' => {
                    if let Some(c) = self.peek() {
                        if c.is_alphabetic() || c == '_' {
                            let first = self.advance().unwrap();
                            let name = self.read_identifier(first);
                            Token::Variable(name)
                        } else {
                            return Err(format!("Invalid variable name at {}:{}", line, col));
                        }
                    } else {
                        return Err(format!("Unexpected $ at end of input"));
                    }
                }

                '\'' | '"' => self.read_string(ch),

                c if c.is_ascii_digit() => self.read_number(c),

                c if c.is_alphabetic() || c == '_' => {
                    let ident = self.read_identifier(c);
                    self.keyword_or_ident(&ident)
                }

                c => return Err(format!("Unexpected character '{}' at {}:{}", c, line, col)),
            };

            tokens.push(self.spanned(token, line, col));
        }
        Ok(tokens)
    }
}
