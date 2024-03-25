mod separators;
pub(crate) mod state;

use crate::char_iterator::CharIteratorReceiver;
use crate::span::Span;

pub struct Lexer<'a> {
    input: CharIteratorReceiver<'a>,
    consumed: String,
    state: state::LexerState,
    current_span: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: CharIteratorReceiver<'a>) -> Self {
        Lexer {
            input,
            consumed: String::new(),
            state: state::LexerState::None,
            current_span: Span::new(0, 0),
        }
    }

    pub fn lex(&mut self) {
        for byte in self.input.by_ref() {
            println!("{}", byte as char);
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
