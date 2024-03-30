use crate::char_iterator::CharIteratorReceiver;
use crate::lexer::separators::Separators;
use crate::lexer::state::LexerState;
use crate::span::Span;
use crate::tokens::ident::Ident;
use crate::tokens::keyword::{Keyword, KeywordType};
use crate::tokens::lit::{Lit, LitKind};
use crate::tokens::punct::{Punct, PunctKind};
use crate::tokens::Token;

mod separators;
pub(crate) mod state;


pub struct LexError {
    span: Span,
    message: String,
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
                self.tokens.push(Punct {
                    kind: p,
                    span: self.current_span.replace(),
                }.into())
            }
            _ => {}
        }


        Ok(())
    }
    fn make_string(&mut self) -> LexResult {
        self.tokens.push(Lit {
            kind: LitKind::String,
            symbol: String::from_utf8(self.consumed.clone()).map_err(|e| LexError {
                span: self.current_span,
                message: e.to_string(),
            })?,
            span: self.current_span.replace(),
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
            self.tokens.push(Keyword {
                ty,
                span: self.current_span.replace(),
            }.into());
            return Ok(());
        }

        //check if symbol is a number
        if symbol.chars().all(|c| c.is_ascii_digit()) {
            self.tokens.push(Lit {
                kind: LitKind::Number,
                symbol,
                span: self.current_span.replace(),
            }.into());
            return Ok(());
        }

        self.tokens.push(Ident {
            ident: symbol,
            span: self.current_span.replace(),
        }.into());
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