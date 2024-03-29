

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexerState {
    None,
    InComment,
    InStringSingle,
    InStringDouble,
    InStringTemplate,
    InChar,
    InNumber,
}
