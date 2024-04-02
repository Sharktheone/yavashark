use std::fmt::Display;
use crate::char_iterator::CharIteratorReceiver;
use crate::lexer::separators::Separators;
use crate::lexer::state::LexerState;
use crate::span::Span;
use crate::tokens::group::Group;
use crate::tokens::ident::Ident;
use crate::tokens::keyword::{Keyword, KeywordType};
use crate::tokens::lit::{Lit, LitKind};
use crate::tokens::punct::{Punct, PunctKind};
use crate::tokens::Token;

mod separators;
pub(crate) mod state;


#[derive(Debug)]
pub struct LexError {
    span: Span,
    message: String,
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}@{}", self.message, self.span)
    }
}

pub type LexResult = Result<(), LexError>;


enum Skip {
    Single(u8),
    Multiple(Vec<u8>),
}

struct Skipper {
    to: Skip,
    hit: usize,
    save: bool,
    save_hit: bool,
}

impl Skipper {
    fn single(to: u8) -> Self {
        Skipper {
            to: Skip::Single(to),
            hit: 0,
            save: true,
            save_hit: false,
        }
    }

    fn single_no_save(to: u8) -> Self {
        Skipper {
            to: Skip::Single(to),
            hit: 0,
            save: false,
            save_hit: false,
        }
    }

    fn multiple(to: Vec<u8>) -> Self {
        Skipper {
            to: Skip::Multiple(to),
            hit: 0,
            save: true,
            save_hit: false,
        }
    }

    fn multiple_no_save(to: Vec<u8>) -> Self {
        Skipper {
            to: Skip::Multiple(to),
            hit: 0,
            save: false,
            save_hit: false,
        }
    }

    fn next(&mut self) -> u8 {
        match &self.to {
            Skip::Single(c) => *c,
            Skip::Multiple(v) => v[self.hit],
        }
    }

    fn hit(&mut self) -> bool {
        self.hit += 1;
        match &self.to {
            Skip::Single(_) => self.hit >= 1,
            Skip::Multiple(v) => self.hit >= v.len(),
        }
    }
}

struct InternalLexer {
    consumed: Vec<u8>,
    state: LexerState,
    current_span: Span,
    tokens: Vec<Token>,
    skipper: Option<Skipper>,
    groups: Vec<Group>,
}

pub struct Lexer<'a> {
    input: CharIteratorReceiver<'a>,
    internal: InternalLexer,
}

impl<'a> Lexer<'a> {
    pub fn new(input: CharIteratorReceiver<'a>) -> Self {
        Lexer {
            input,
            internal: InternalLexer {
                consumed: Vec::with_capacity(16),
                state: LexerState::None,
                current_span: Span::new(0, 0),
                tokens: Vec::with_capacity(1024),
                skipper: None,
                groups: Vec::with_capacity(8),
            },
        }
    }

    pub fn lex(&mut self) -> LexResult {
        let input = &mut self.input;
        let int = &mut self.internal;

        for c in input {
            if let Some(skipper) = &mut int.skipper {
                if c != skipper.next() {
                    if skipper.save_hit {
                        int.consumed.push(c);
                    }
                    if skipper.hit() {
                        int.skipper = None;
                    }
                    continue;
                } else if skipper.save {
                    int.consumed.push(c);
                }
            }
            int.lex_char(c)?;
        };

        Ok(())
    }
}


impl InternalLexer {
    fn lex_char(&mut self, c: u8) -> LexResult {
        match Separators::from_u8(c) {
            Some(sep) => {
                self.separator(sep)?;
            }
            None => {
                self.consumed.push(c);
            }
        }

        Ok(())
    }

    fn separator(&mut self, sep: Separators) -> LexResult {
        match sep {
            Separators::Quote => {
                if self.consumed.last() == Some(&b'\\') {
                    self.consumed.pop();
                    return Ok(());
                }

                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringSingle;
                        self.skipper = Some(Skipper::single(b'\''));
                    }
                }
            }
            Separators::DoubleQuote => {
                if self.consumed.last() == Some(&b'\\') {
                    self.consumed.pop();
                    return Ok(());
                }

                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringDouble;
                        self.skipper = Some(Skipper::single(b'"'));
                    }
                }
            }
            Separators::Backtick => {
                if self.consumed.last() == Some(&b'\\') {
                    self.consumed.pop();
                    return Ok(());
                }

                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringTemplate;
                        self.skipper = Some(Skipper::single(b'`'));
                    }
                }
            }
            Separators::Punct(p) => {
                self.check_consumed()?;

                match &p {
                    PunctKind::Slash => {
                        if self.consumed.last() == Some(&b'/') {
                            self.consumed.pop();
                            self.skipper = Some(Skipper::single_no_save(b'\n'));
                        }
                    }
                    PunctKind::Asterisk => {
                        if self.consumed.last() == Some(&b'/') {
                            self.consumed.pop();
                            self.skipper = Some(Skipper::multiple_no_save(vec![b'*', b'/']));
                        }
                    }
                    _ => {}
                }
                let span = self.current_span.replace();
                self.push_token(Punct {
                    kind: p,
                    span,
                }.into())
            }
            Separators::ParenthesesOpen => {
                self.check_consumed()?;
                
                self.groups.push(Group::paren(self.current_span))
            }
            Separators::ParenthesesClose => {
                self.check_consumed()?;
                
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_paren() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!("Expected `)` found `{}`", group.delimiter.get_closing()),
                        });
                    }
                    
                    group.update_span_end(self.current_span.end);


                    self.push_token(group.into());
                }
            }
            Separators::CurlyBraceOpen => {
                self.check_consumed()?;
                
                self.groups.push(Group::brace(self.current_span))
            }
            Separators::CurlyBraceClose => {
                self.check_consumed()?;
                
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_brace() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!("Expected `{}` found `}}`", group.delimiter.get_closing()),
                        });
                    }
                    
                    group.update_span_end(self.current_span.end);
                    
                    self.push_token(group.into());
                }
            }
            Separators::BracketOpen => {
                self.check_consumed()?;
                
                self.groups.push(Group::bracket(self.current_span))
            }
            Separators::BracketClose => {
                self.check_consumed()?;
                
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_bracket() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!("Expected `{}` found `]`", group.delimiter.get_closing()),
                        });
                    }
                    
                    group.update_span_end(self.current_span.end);
                    
                    self.push_token(group.into());
                }
            }
            Separators::AngleBracketOpen => {
                self.check_consumed()?;
                
                self.groups.push(Group::angle_bracket(self.current_span))
            }
            Separators::AngleBracketClose => {
                self.check_consumed()?;
                
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_angle_bracket() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!("Expected `{}` found `>`", group.delimiter.get_closing()),
                        });
                    }
                    
                    group.update_span_end(self.current_span.end);
                    
                    self.push_token(group.into());
                }
            }
            _ => {} // ignore Space, NewLine, Tab
        }


        Ok(())
    }
    fn make_string(&mut self) -> LexResult {
        let span = self.current_span.replace();
        self.push_token(Lit {
            kind: LitKind::String,
            symbol: String::from_utf8(self.consumed.clone()).map_err(|e| LexError {
                span: self.current_span,
                message: e.to_string(),
            })?,
            span,
        }.into());

        Ok(())
    }

    fn check_consumed(&mut self) -> LexResult {
        if self.consumed.is_empty() {
            return Ok(());
        }

        let symbol = String::from_utf8(self.consumed.clone()).map_err(|e| LexError {
            span: self.current_span,
            message: e.to_string(),
        })?;
        self.consumed.clear();


        if let Some(ty) = KeywordType::from_string(&symbol) {
            let span =self.current_span.replace();
            self.push_token(Keyword {
                ty,
                span,
            }.into());
            return Ok(());
        }

        //check if symbol is a number
        if symbol.chars().all(|c| c.is_ascii_digit()) {
            let span = self.current_span.replace();
            self.push_token(Lit {
                kind: LitKind::Number,
                symbol,
                span,
            }.into());
            return Ok(());
        }

        let span = self.current_span.replace();
        self.push_token(Ident {
            ident: symbol,
            span,
        }.into());
        Ok(())
    }
    
    fn push_token(&mut self, token: Token) {
        if let Some(group) = self.groups.last_mut() {
            group.push(token);
        } else {
            self.tokens.push(token);
        }
    }
}


impl<'a> TryFrom<String> for Lexer<'a> {
    type Error = anyhow::Error;
    fn try_from(s: String) -> anyhow::Result<Self> {
        Ok(Self::new(CharIteratorReceiver::try_from(s)?))
    }
}

impl<'a> TryFrom<&str> for Lexer<'a> {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> anyhow::Result<Self> {
        Ok(Self::new(CharIteratorReceiver::try_from(s)?))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let mut lexer = Lexer::try_from("let a = 1;").unwrap();
        lexer.lex().unwrap();
        let tokens = lexer.internal.tokens;
        println!("{:?}", tokens);
    }
}