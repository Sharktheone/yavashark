use crate::tokens::ident::Ident;
use crate::tokens::keyword::Keyword;
use crate::tokens::lit::Lit;
use crate::tokens::punct::Punct;

pub mod ident;
pub mod keyword;
pub mod lit;
pub mod punct;

pub enum Token {
    Ident(Ident),
    Lit(Lit),
    Punct(Punct),
    Keyword(Keyword),
}


impl From<Ident> for Token {
    fn from(ident: Ident) -> Self {
        Token::Ident(ident)
    }
}

impl From<Lit> for Token {
    fn from(lit: Lit) -> Self {
        Token::Lit(lit)
    }
}

impl From<Punct> for Token {
    fn from(punct: Punct) -> Self {
        Token::Punct(punct)
    }
}

impl From<Keyword> for Token {
    fn from(keyword: Keyword) -> Self {
        Token::Keyword(keyword)
    }
}

impl Token {
    pub fn is_ident(&self) -> bool {
        matches!(self, Token::Ident(_))
    }

    pub fn is_lit(&self) -> bool {
        matches!(self, Token::Lit(_))
    }

    pub fn is_punct(&self) -> bool {
        matches!(self, Token::Punct(_))
    }

    pub fn is_keyword(&self) -> bool {
        matches!(self, Token::Keyword(_))
    }
}