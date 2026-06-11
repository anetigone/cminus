use std::error::Error;

use displaydoc::Display;

use crate::lexer::token::Span;

/// 语义分析错误
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum SemanticErrorKind {
    /// Duplicate symbol {name}.
    DuplicateSymbol { name: String },
    /// Undefined symbol {name}.
    UndefinedSymbol { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub span: Span,
}

impl SemanticError {
    pub fn new(kind: SemanticErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Semantic Error at {}] {}", self.span, self.kind)
    }
}

impl Error for SemanticError {}
