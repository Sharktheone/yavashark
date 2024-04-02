use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Punct {
    pub kind: PunctKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunctKind {
    Comma,
    Dot,
    Colon,
    QuestionMark,
    ExclamationMark,
    Semicolon,
    Equal,
    Plus,
    Minus,
    Asterisk,
    Slash,
    And,
    Percent,
    Pipe,
}
