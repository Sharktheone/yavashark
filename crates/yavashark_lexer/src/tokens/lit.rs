use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitKind {
    //probably most of them aren't needed in ts
    Number,
    String,
    StringTemplate,
    Regex,
}
