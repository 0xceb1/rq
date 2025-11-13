use uuid::Uuid;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    BackSlash,
    Star,
    BackTick,   // `
    Apostrophe, // '
    Hash,       // #
    At,         // @
    Tilde,      // ~
    Pipe,       // |
    Ampersand,  // &
    Caret,      // ^
    Question,   // ?
    Dollar,     // $
    Underscore, // _
    Bang,

    // One or two character tokens.
    NotEqual, // <>
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Colon,
    ColonColon,

    // Literals.
    Identifier,
    Guid(Uuid),
    Byte(u8),
    Char(char),
    Symbol(usize),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Date(i32), // 2000.01.01 = 0
    Month(i32),
    Minute(i32),
    Second(i32),
    // Time(i32),      // hh:mm:ss.uuu
    Timespan(i64),  // hh:mm:ss.nnnnnnnnn
    Timestamp(i64), // YYYY.MM.DDDhh:mm:ss.nnnnnnnn

    // // Keywords.
    // And,
    // Class,
    // Else,
    // False,
    // Fun,
    // For,
    // If,
    // Nil,
    // Or,
    // Print,
    // Return,
    // Super,
    // This,
    // True,
    // Var,
    // While,
    // Lambda,
    Eof,
}
