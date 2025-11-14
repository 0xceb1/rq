use lasso::{Spur, ThreadedRodeo};
use std::fmt;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(Spur);

static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::default);

impl Symbol {
    pub fn from(literal: &str) -> Self {
        Self(INTERNER.get_or_intern(literal))
    }

    pub fn resolve(&self) -> &str {
        INTERNER.resolve(&self.0)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}", self.resolve())
    }
}

impl From<&str> for Symbol {
    fn from(literal: &str) -> Self {
        Self(INTERNER.get_or_intern(literal))
    }
}

impl From<String> for Symbol {
    fn from(literal: String) -> Self {
        Self(INTERNER.get_or_intern(literal))
    }
}

impl From<Symbol> for String {
    fn from(symbol: Symbol) -> Self {
        String::from(symbol.resolve())
    }
}
