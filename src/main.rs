use miette::Result;
use rq::chrono;
use rq::{Lexer, Token, TokenKind};

fn main() -> Result<()> {
    // let code = "\"a\"\"中\"\"This is a string with escaped \\\"values\\\"\"";
    let code = "```````b```1`";
    println!("{}", code);
    let lexer = Lexer::new(code);
    for c in lexer {
        println!("{}", c?);
    }
    Ok(())
}
