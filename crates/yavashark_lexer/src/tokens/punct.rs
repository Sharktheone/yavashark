use crate::span::Span;

pub struct Punct {
    pub kind: PunctKind,
    pub span: Span,
}

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
    BracketOpen,
    BracketClose,
    ParenthesesOpen,
    ParenthesesClose,
    CurlyBraceOpen,
    CurlyBraceClose,
    AngleBracketOpen,
    AngleBracketClose,
}
