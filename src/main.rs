use miette::Result;
use rq::chrono;
use rq::{Lexer, Token, TokenKind};

fn main() -> Result<()> {
    // let code = "\"a\"\"中\"\"This is a string with escaped \\\"values\\\"\"";
    // let code = "`0 `1";
    let code =
        "* /this is a comment\n`s1 /this is a multiline comment\nthis is a multiline comment\\`s2";
    println!("{code}");
    let lexer = Lexer::new(code);
    for c in lexer {
        println!("{}", c?);
    }
    Ok(())
}
