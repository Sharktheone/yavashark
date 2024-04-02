use crate::span::Span;
use crate::tokens::Token;

pub struct Group {
    pub delimiter: Delimiter,
    pub tokens: Vec<Token>,
    pub span: Span,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    AngleBracket,
}


impl Group {
    pub fn new(delimiter: Delimiter, tokens: Vec<Token>, span: Span) -> Self {
        Group {
            delimiter,
            tokens,
            span,
        }
    }
    
    pub fn paren(span: Span) -> Self {
        Group::new(Delimiter::Parenthesis, Vec::new(), span)
    }
    
    pub fn brace(span: Span) -> Self {
        Group::new(Delimiter::Brace, Vec::new(), span)
    }
    
    pub fn bracket(span: Span) -> Self {
        Group::new(Delimiter::Bracket, Vec::new(), span)
    }
    
    pub fn angle_bracket(span: Span) -> Self {
        Group::new(Delimiter::AngleBracket, Vec::new(), span)
    }
    
    pub fn update_span_end(&mut self, end: usize) {
        self.span.end = end;
    }
    
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
    
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    
    pub fn is_paren(&self) -> bool {
        matches!(self.delimiter, Delimiter::Parenthesis)
    }
    
    pub fn is_brace(&self) -> bool {
        matches!(self.delimiter, Delimiter::Brace)
    }
    
    pub fn is_bracket(&self) -> bool {
        matches!(self.delimiter, Delimiter::Bracket)
    }
    
    pub fn is_angle_bracket(&self) -> bool {
        matches!(self.delimiter, Delimiter::AngleBracket)
    }
}


impl Delimiter {
    pub fn get_closing(&self) -> char {
        match self {
            Delimiter::Parenthesis => ')',
            Delimiter::Brace => '}',
            Delimiter::Bracket => ']',
            Delimiter::AngleBracket => '>',
        }
    }
}