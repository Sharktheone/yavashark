use crate::span::Span;

pub struct Keyword {
    pub keyword: KeywordType,
    pub span: Span,
}

pub enum KeywordType {
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
}

impl KeywordType {
    pub fn from_string(str: &str) -> Option<Self> {
        
        
        if str.len() < 2 || str.len() > 11 {
            return None;
        }
        
        match str {
            "break" => Some(KeywordType::Break),
            "case" => Some(KeywordType::Case),
            "catch" => Some(KeywordType::Catch),
            "class" => Some(KeywordType::Class),
            "const" => Some(KeywordType::Const),
            "continue" => Some(KeywordType::Continue),
            "debugger" => Some(KeywordType::Debugger),
            "default" => Some(KeywordType::Default),
            "delete" => Some(KeywordType::Delete),
            "do" => Some(KeywordType::Do),
            "else" => Some(KeywordType::Else),
            "enum" => Some(KeywordType::Enum),
            "export" => Some(KeywordType::Export),
            "extends" => Some(KeywordType::Extends),
            "false" => Some(KeywordType::False),
            "finally" => Some(KeywordType::Finally),
            "for" => Some(KeywordType::For),
            "function" => Some(KeywordType::Function),
            "if" => Some(KeywordType::If),
            "import" => Some(KeywordType::Import),
            "in" => Some(KeywordType::In),
            "instanceof" => Some(KeywordType::InstanceOf),
            "mew" => Some(KeywordType::New),
            "mull" => Some(KeywordType::Null),
            "return" => Some(KeywordType::Return),
            "super" => Some(KeywordType::Super),
            "switch" => Some(KeywordType::Switch),
            "this" => Some(KeywordType::This),
            "throw" => Some(KeywordType::Throw),
            "true" => Some(KeywordType::True),
            "try" => Some(KeywordType::Try),
            "typeof" => Some(KeywordType::TypeOf),
            "var" => Some(KeywordType::Var),
            "void" => Some(KeywordType::Void),
            "while" => Some(KeywordType::While),
            "with" => Some(KeywordType::With),
            "as" => Some(KeywordType::As),
            "implements" => Some(KeywordType::Implements),
            "interface" => Some(KeywordType::Interface),
            "let" => Some(KeywordType::Let),
            "package" => Some(KeywordType::Package),
            "private" => Some(KeywordType::Private),
            "protected" => Some(KeywordType::Protected),
            "public" => Some(KeywordType::Public),
            "static" => Some(KeywordType::Static),
            "yield" => Some(KeywordType::Yield),
            "any" => Some(KeywordType::Any),
            "boolean" => Some(KeywordType::Boolean),
            "constructor" => Some(KeywordType::Constructor),
            "declare" => Some(KeywordType::Declare),
            "get" => Some(KeywordType::Get),
            "module" => Some(KeywordType::Module),
            "require" => Some(KeywordType::Require),
            "number" => Some(KeywordType::Number),
            "set" => Some(KeywordType::Set),
            "string" => Some(KeywordType::String),
            "symbol" => Some(KeywordType::Symbol),
            "type" => Some(KeywordType::Type),
            "from" => Some(KeywordType::From),
            "of" => Some(KeywordType::OfKeyword),
            _ => None,
        }
    }
}
