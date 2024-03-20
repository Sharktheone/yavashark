use crate::span::Span;
use crate::tokens::Token;

pub struct Group {
    pub delimiter: Delimiter,
    pub tokens: Vec<Token>,
    pub span: Span,
}

pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}
