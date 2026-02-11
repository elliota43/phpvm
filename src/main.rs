mod lexer;
mod token;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {

    let source = r#"<?php
$x = 10;
$y = 20;
echo $x + $y;

if ($x > $y) {
echo "x is bigger";
} else {
    echo "y is bigger";
}

function add($a, $b) {
    return $a + $b;
}

echo add(5, 3);
"#;

    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("Lexer err: {}", e); return; }
    };

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            for stmt in &ast {
                println!("{:#?}", stmt);
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
