use crate::span::Span;

pub struct Punct {
    pub kind: PunctKind,
    pub span: Span,
}

pub enum PunctKind {
    Colon,
    Semicolon,
    Comma,
    // ...
}