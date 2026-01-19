use crate::lex::Lexer;
use crate::qtype::Q;

pub struct Parser<'de> {
    source: &'de str,
    lexer: Lexer<'de>,
}

// impl<'de> Parser<'de> {
//     pub fn new(input: &'de str) -> Result<Self, Error> {
//         Ok(Self {
//             tokens: preprocess(input)?,
//         })
//     }
// }

#[derive(Debug, Clone)]
pub enum Expr {
    Identifier,
    Atom(Q),
    Vector(Vec<Q>),  // homogeneous list
    List(Vec<Expr>), // heterogeneous/nested list
}
