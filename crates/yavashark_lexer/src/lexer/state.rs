pub(crate) enum LexerState {
    None,
    InComment,
    InString,
    InChar,
    InNumber,
}
