use crate::char_iterator::CharIteratorReceiver;
use crate::lexer::separators::Separators;
use crate::lexer::state::LexerState;
use crate::span::Span;
use crate::tokens::lit::{Lit, LitKind};
use crate::tokens::punct::Punct;
use crate::tokens::Token;

mod separators;
pub(crate) mod state;


struct LexError {
    span: Span,
    message: String,
}

type LexResult = Result<(), LexError>;

struct InternalLexer {
    consumed: Vec<u8>,
    state: LexerState,
    current_span: Span,

    tokens: Vec<Token>,
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
            },
        }
    }

    pub fn lex(&mut self) {
        let input = &mut self.input;
        let int = &mut self.internal;

        for c in input {
            int.lex_char(c);
        };
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
                    LexerState::InStringSingle | LexerState::InStringDouble => {
                        self.consumed.push(b'\'');
                    }
                    LexerState::None => {
                        self.state = LexerState::InStringSingle;
                    }
                    _ => {}
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
                    LexerState::InStringSingle | LexerState::InStringDouble => {
                        self.consumed.push(b'"');
                    }
                    LexerState::None => {
                        self.state = LexerState::InStringDouble;
                    }
                    _ => {}
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
                    LexerState::InStringSingle | LexerState::InStringDouble => {
                        self.consumed.push(b'`');
                    }
                    LexerState::None => {
                        self.state = LexerState::InStringTemplate;
                    }
                    _ => {}
                }
            }
            Separators::Punct(p) => {
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