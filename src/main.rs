use miette::Result;
use rq::chrono;
use rq::{Lexer, Literal, Token, TokenKind};

fn main() -> Result<()> {
    let code = "\"This is a string with escaped \\\"values\\\"\"";
    println!("{}", code);
    let lexer = Lexer::new(code);
    for c in lexer {
        println!("{}", c?);
    }
    Ok(())
}
