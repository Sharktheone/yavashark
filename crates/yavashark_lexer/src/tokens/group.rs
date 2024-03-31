use crate::tokens::Token;

pub struct Group {
    pub delimiter: Delimiter,
    pub stream: Box<Token>,
}

pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    AngleBracket,
}