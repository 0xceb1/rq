use miette::Result;
use rq::chrono;
use rq::{Lexer, Literal, Token, TokenKind};

fn main() -> Result<()> {
    let code = "*&#!@中";
    let lexer = Lexer::new(code);
    for c in lexer {
        println!("{:?}", c?);
    }
    Ok(())
}
