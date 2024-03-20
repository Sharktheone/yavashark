use crate::span::Span;

pub struct Keyword {
    pub keyword: KeywordType,
    pub span: Span,
}

enum KeywordType {
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    InstanceOf,
    New,
    Null,
    Return,
    Super,
    Switch,
    This,
    Throw,
    True,
    Try,
    TypeOf,
    Var,
    Void,
    While,
    With,
    // Strict mode reserved words
    As,
    Implements,
    Interface,
    Let,
    Package,
    Private,
    Protected,
    Public,
    Static,
    Yield,
    // Contextual keywords
    Any,
    Boolean,
    Constructor,
    Declare,
    Get,
    Module,
    Require,
    Number,
    Set,
    String,
    Symbol,
    Type,
    From,
    OfKeyword,
    Invalid,
}
