use miette::{IntoDiagnostic, Result};
use rq::Parser;
use std::io::{self, Write};

fn main() -> Result<()> {
    loop {
        print!("q) ");
        io::stdout().flush().into_diagnostic()?;
        let mut buf = String::new();
        let bytes_read = io::stdin().read_line(&mut buf).into_diagnostic()?;
        if bytes_read == 0 {
            break;
        }
        if !buf.trim().is_empty() {
            let mut parser = Parser::new(buf.as_str());
            match parser.parse() {
                Ok(expr) => println!("{}", expr),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
    Ok(())
}
