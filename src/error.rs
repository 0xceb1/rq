use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// A q error such as `'type` or `'length`, shown the way q names them.
/// Evaluation raises it without a location; [`QError::at`] attaches the source and span.
#[derive(Diagnostic, Debug, Error, PartialEq)]
#[error("'{name}")]
pub struct QError {
    pub name: &'static str,
    pub label: String,
    #[label("{label}")]
    pub span: Option<SourceSpan>,
    #[source_code]
    src: String,
}

impl QError {
    fn new(name: &'static str, label: impl Into<String>) -> Self {
        Self {
            name,
            label: label.into(),
            span: None,
            src: String::new(),
        }
    }

    pub fn length(lhs: usize, rhs: usize) -> Self {
        Self::new("length", format!("lhs has {lhs} items, rhs has {rhs}"))
    }

    pub fn type_(label: impl Into<String>) -> Self {
        Self::new("type", label)
    }

    /// Not yet implemented: valid q that rq does not support yet.
    pub fn nyi(label: impl Into<String>) -> Self {
        Self::new("nyi", label)
    }

    pub fn at(self, src: &str, span: impl Into<SourceSpan>) -> Self {
        Self {
            span: Some(span.into()),
            src: src.to_string(),
            ..self
        }
    }
}
