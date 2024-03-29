use crate::char_iterator::CharIteratorReceiver;
use crate::span::Span;

mod separators;
pub(crate) mod state;


struct InternalLexer {
    consumed: String,
    state: state::LexerState,
    current_span: Span,
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
                consumed: String::new(),
                state: state::LexerState::None,
                current_span: Span::new(0, 0),
            },
        }
    }

    pub fn lex(&mut self) {
        let input = &mut self.input;
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