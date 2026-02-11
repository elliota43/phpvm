mod lexer;
mod token;

use lexer::Lexer;

fn main() {
    let source = r#"<?php
echo 1 + 2;
$name = "world";
echo "hello " . $name;
"#;
    let mut lexer = Lexer::new(source);
    match lexer.tokenize() {
        Ok(tokens) => {
            for t in &tokens {
                println!("{:>3}:{:<3} {:?}", t.line, t.col, t.token);
            }
        }
        Err(e) => {
            eprintln!("Lexer error: {}", e);
        }
    }
}
