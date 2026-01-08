use miette::{IntoDiagnostic, Result};
use rq::Lexer;
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
            let lexer = Lexer::new(buf.as_str());
            for c in lexer {
                match c {
                    Ok(token) => println!("{}", token),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
