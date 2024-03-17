use crate::span::Span;

pub struct Lit {
    pub kind: LitKind,
    pub symbol: String,
    pub span: Span,
}

pub enum LitKind { //probably most of them aren't needed in ts
    Number,
    String,
}