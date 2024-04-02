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
    hit_fn: Option<fn(&mut InternalLexer) -> LexResult>
}

impl Skipper {
    fn single(to: u8) -> Self {
        Skipper {
            to: Skip::Single(to),
            hit: 0,
            save: true,
            save_hit: false,
            hit_fn: None,
        }
    }

    fn single_no_save(to: u8) -> Self {
        Skipper {
            to: Skip::Single(to),
            hit: 0,
            save: false,
            save_hit: false,
            hit_fn: None,
        }
    }

    fn multiple(to: Vec<u8>) -> Self {
        Skipper {
            to: Skip::Multiple(to),
            hit: 0,
            save: true,
            save_hit: false,
            hit_fn: None,
        }
    }

    fn multiple_no_save(to: Vec<u8>) -> Self {
        Skipper {
            to: Skip::Multiple(to),
            hit: 0,
            save: false,
            save_hit: false,
            hit_fn: None,
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
                if c == skipper.next() {
                    if skipper.save_hit {
                        int.consumed.push(c);
                    }
                    if skipper.hit() {
                        if let Some(hit_fn) = skipper.hit_fn {
                            hit_fn(int)?;
                        }
                        int.skipper = None;
                        int.current_span.start += 1;
                    }
                } else if skipper.save {
                    int.consumed.push(c);
                }
                int.current_span.extend();
                continue
            }
            int.lex_char(c)?;
            int.current_span.extend();
        }

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
        self.check_consumed()?;
        match sep {
            Separators::Quote => {
                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringSingle;
                        let mut skip = Skipper::single(b'\'');
                        skip.hit_fn = Some(InternalLexer::make_str_lit);
                        self.skipper = Some(skip);
                    }
                }
            }
            Separators::DoubleQuote => {
                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringDouble;
                        let mut skip = Skipper::single(b'"');
                        skip.hit_fn = Some(InternalLexer::make_str_lit);
                        self.skipper = Some(skip);
                    }
                }
            }
            Separators::Backtick => {
                match self.state {
                    LexerState::InStringTemplate => {
                        self.state = LexerState::None;
                        self.make_string()?;
                    }
                    _ => {
                        self.state = LexerState::InStringTemplate;
                        let mut skip = Skipper::single(b'`');
                        skip.hit_fn = Some(InternalLexer::make_str_lit);
                        self.skipper = Some(skip);
                    }
                }
            }
            Separators::Punct(p) => {
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
                self.push_token(Punct { kind: p, span }.into());
            }
            Separators::ParenthesesOpen => {
                self.groups.push(Group::paren(self.current_span))
            }
            Separators::ParenthesesClose => {
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_paren() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!(
                                "Expected `)` found `{}`",
                                group.delimiter.get_closing()
                            ),
                        });
                    }

                    group.update_span_end(self.current_span.end);

                    self.push_token(group.into());
                }
            }
            Separators::CurlyBraceOpen => {
                self.groups.push(Group::brace(self.current_span))
            }
            Separators::CurlyBraceClose => {
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_brace() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!(
                                "Expected `{}` found `}}`",
                                group.delimiter.get_closing()
                            ),
                        });
                    }

                    group.update_span_end(self.current_span.end);

                    self.push_token(group.into());
                }
            }
            Separators::BracketOpen => {
                self.groups.push(Group::bracket(self.current_span))
            }
            Separators::BracketClose => {
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_bracket() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!(
                                "Expected `{}` found `]`",
                                group.delimiter.get_closing()
                            ),
                        });
                    }

                    group.update_span_end(self.current_span.end);

                    self.push_token(group.into());
                }
            }
            Separators::AngleBracketOpen => {
                self.groups.push(Group::angle_bracket(self.current_span))
            }
            Separators::AngleBracketClose => {
                if let Some(mut group) = self.groups.pop() {
                    if !group.is_angle_bracket() {
                        return Err(LexError {
                            span: self.current_span,
                            message: format!(
                                "Expected `{}` found `>`",
                                group.delimiter.get_closing()
                            ),
                        });
                    }

                    group.update_span_end(self.current_span.end);

                    self.push_token(group.into());
                }
            }
            _ => {
                self.current_span.reset();
            } // ignore Space, NewLine, Tab
        }

        self.current_span.start += 1;

        Ok(())
    }
    fn make_string(&mut self) -> LexResult {
        let span = self.current_span.replace();
        self.push_token(
            Lit {
                kind: LitKind::String,
                symbol: String::from_utf8(self.consumed.clone()).map_err(|e| LexError {
                    span: self.current_span,
                    message: e.to_string(),
                })?,
                span,
            }
                .into(),
        );

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
            let span = self.current_span.replace_dec();
            self.push_token(Keyword { ty, span }.into());
            return Ok(());
        }

        //check if symbol is a number
        if symbol.chars().all(|c| c.is_ascii_digit()) {
            let span = self.current_span.replace_dec();
            self.push_token(
                Lit {
                    kind: LitKind::Number,
                    symbol,
                    span,
                }
                    .into(),
            );
            return Ok(());
        }

        let span = self.current_span.replace_dec();
        self.push_token(
            Ident {
                ident: symbol,
                span,
            }
                .into(),
        );
        Ok(())
    }

    fn push_token(&mut self, token: Token) {
        if let Some(group) = self.groups.last_mut() {
            group.push(token);
        } else {
            self.tokens.push(token);
        }
    }

    fn make_str_lit(&mut self) -> LexResult {
        let mut span = self.current_span.replace();
        span.end -= 1;
        self.push_token(
            Lit {
                kind: LitKind::String,
                symbol: String::from_utf8(self.consumed.clone()).map_err(|e| LexError {
                    span,
                    message: e.to_string(),
                })?,
                span,
            }
                .into(),
        );
        self.consumed.clear();

        Ok(())

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
    fn simple() {
        let mut lexer = Lexer::try_from("let a = 1;").unwrap();
        lexer.lex().unwrap();
        let tokens = lexer.internal.tokens;
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Keyword(Keyword {
            ty: KeywordType::Let,
            span: Span::new(0, 2),
        }));
        assert_eq!(tokens[1], Token::Ident(Ident {
            ident: "a".to_string(),
            span: Span::new(4, 4),
        }));
        assert_eq!(tokens[2], Token::Punct(Punct {
            kind: PunctKind::Equal,
            span: Span::new(6, 6),
        }));
        assert_eq!(tokens[3], Token::Lit(Lit {
            kind: LitKind::Number,
            symbol: "1".to_string(),
            span: Span::new(8, 8),
        }));
        assert_eq!(tokens[4], Token::Punct(Punct {
            kind: PunctKind::Semicolon,
            span: Span::new(9, 9),
        }));
    }

    #[test]
    fn string() {
        let mut lexer = Lexer::try_from(r#"let a = "hello";"#).unwrap();
        lexer.lex().unwrap();
        let tokens = lexer.internal.tokens;
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Keyword(Keyword {
            ty: KeywordType::Let,
            span: Span::new(0, 2),
        }));
        assert_eq!(tokens[1], Token::Ident(Ident {
            ident: "a".to_string(),
            span: Span::new(4, 4),
        }));
        assert_eq!(tokens[2], Token::Punct(Punct {
            kind: PunctKind::Equal,
            span: Span::new(6, 6),
        }));
        assert_eq!(tokens[3], Token::Lit(Lit {
            kind: LitKind::String,
            symbol: "hello".to_string(),
            span: Span::new(9, 13),
        }));
        assert_eq!(tokens[4], Token::Punct(Punct {
            kind: PunctKind::Semicolon,
            span: Span::new(15, 15),
        }));
    }
}
