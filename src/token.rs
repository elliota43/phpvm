#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    StringLiteral(String),

    // Identifiers & keywords
    Variable(String),
    Identifier(String),

    // keywords
    Echo,
    If,
    Else,
    Elseif,
    While,
    For,
    Function,
    Return,
    True,
    False,
    Null,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Dot,          // string concat
    Assign,       // =
    Equal,        // ==
    Identical,    // ===
    NotEqual,     // !=
    NotIdentical, // !==
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And, // &&
    Or,  // ||
    Not, // !

    // Delimiters
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Comma,
    Arrow,

    // Special
    OpenTag, // <?php
    Eof,
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
