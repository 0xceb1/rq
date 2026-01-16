use crate::lex::{Lexer, Numerical, Token, TokenKind};
use crate::qtype::Q;
use miette::{Diagnostic, Error, SourceSpan};
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
#[error("Incompatible type in vector")]
pub struct InvalidVectorLiteralError {
    #[source_code]
    src: String,

    #[label = "this element"]
    err_span: SourceSpan,

    #[help]
    help: Option<String>,
}

impl InvalidVectorLiteralError {
    pub fn line(&self) -> usize {
        let until_unrecongized = &self.src[..=self.err_span.offset()];
        until_unrecongized.lines().count()
    }
}

#[derive(Debug, Clone)]
pub enum PreToken<'de> {
    Single(Token<'de>),
    String(Token<'de>),
    ByteVec(Token<'de>),
    SymbolVec(Token<'de>),
    Vector {
        tokens: Vec<Token<'de>>,
        elem_type: Numerical,
    },
}

fn is_numeric(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::Typed(_) | TokenKind::Untyped(_))
}

fn is_typed(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::Typed(_))
}

fn is_adjacent(prev: &Token, next: &Token) -> bool {
    prev.offset + prev.origin.len() == next.offset
}

pub fn preprocess(input: &str) -> Result<Vec<PreToken<'_>>, Error> {
    // TODO: make this lazy
    let tokens: Vec<Token> = Lexer::new(input).collect::<Result<_, _>>()?;
    let mut result: Vec<PreToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let tok = tokens[i];

        if let Some(pretoken) = match tok.kind {
            TokenKind::String => Some(PreToken::String(tok)),
            TokenKind::ByteVec => Some(PreToken::ByteVec(tok)),
            TokenKind::SymbolVec => Some(PreToken::SymbolVec(tok)),
            _ if !is_numeric(tok.kind) => Some(PreToken::Single(tok)),
            _ => None,
        } {
            result.push(pretoken);
            i += 1;
            continue;
        }

        let mut group = vec![tok];
        while i + group.len() < tokens.len() {
            let next = tokens[i + group.len()];
            if is_numeric(next.kind) && is_adjacent(group.last().unwrap(), &next) {
                group.push(next);
            } else {
                break;
            }
        }
        i += group.len();

        if group.len() == 1 {
            result.push(PreToken::Single(group[0]));
        } else {
            if let Some(illegal) = group[..group.len() - 1].iter().find(|t| is_typed(t.kind)) {
                return Err(InvalidVectorLiteralError {
                    src: input.to_string(),
                    err_span: SourceSpan::from(
                        illegal.offset..illegal.offset + illegal.origin.len(),
                    ),
                    help: Some("only the last element is allowed to have a suffix".into()),
                }
                .into());
            }

            let last = group.last().unwrap();
            let elem_type = match last.kind {
                TokenKind::Typed(t) | TokenKind::Untyped(t) => t,
                _ => unreachable!(),
            };
            result.push(PreToken::Vector {
                tokens: group,
                elem_type,
            });
        }
    }

    Ok(result)
}

pub struct Parser<'de> {
    tokens: Vec<PreToken<'de>>,
}

impl<'de> Parser<'de> {
    pub fn new(input: &'de str) -> Result<Self, Error> {
        Ok(Self {
            tokens: preprocess(input)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Identifier,
    Atom(Q),
    Vector(Vec<Q>),  // homogeneous list
    List(Vec<Expr>), // heterogeneous/nested list
}
