//Separators are characters that are used to separate tokens in the source code. Such as keywords, identifiers, literals, etc.
pub(crate) enum Separators {
    Space,
    NewLine,
    Tab,
    Comma,
    Dot,
    Colon,
    QuestionMark,
    ExclamationMark,
    Semicolon,
    Equal,
    Plus,
    Minus,
    Asterisk,
    Slash,
    And,
    Percent,
    Pipe,
    Quote,
    DoubleQuote,
    Backtick,
    BracketOpen,
    BracketClose,
    ParenthesesOpen,
    ParenthesesClose,
    CurlyBraceOpen,
    CurlyBraceClose,
    AngleBracketOpen,
    AngleBracketClose,
}

impl Separators {
    pub(crate) fn from_char(c: char) -> Option<Self> {
        match c {
            ' ' => Some(Self::Space),
            '\n' => Some(Self::NewLine),
            '\t' => Some(Self::Tab),
            ',' => Some(Self::Comma),
            '.' => Some(Self::Dot),
            ':' => Some(Self::Colon),
            '?' => Some(Self::QuestionMark),
            '!' => Some(Self::ExclamationMark),
            ';' => Some(Self::Semicolon),
            '=' => Some(Self::Equal),
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            '*' => Some(Self::Asterisk),
            '/' => Some(Self::Slash),
            '&' => Some(Self::And),
            '%' => Some(Self::Percent),
            '|' => Some(Self::Pipe),
            '\'' => Some(Self::Quote),
            '"' => Some(Self::DoubleQuote),
            '`' => Some(Self::Backtick),
            '[' => Some(Self::BracketOpen),
            ']' => Some(Self::BracketClose),
            '(' => Some(Self::ParenthesesOpen),
            ')' => Some(Self::ParenthesesClose),
            '{' => Some(Self::CurlyBraceOpen),
            '}' => Some(Self::CurlyBraceClose),
            '<' => Some(Self::AngleBracketOpen),
            '>' => Some(Self::AngleBracketClose),
            _ => None,
        }
    }

    pub(crate) fn from_u8(c: u8) -> Option<Self> {
        match c {
            b' ' => Some(Self::Space),
            b'\n' => Some(Self::NewLine),
            b'\t' => Some(Self::Tab),
            b',' => Some(Self::Comma),
            b'.' => Some(Self::Dot),
            b':' => Some(Self::Colon),
            b'?' => Some(Self::QuestionMark),
            b'!' => Some(Self::ExclamationMark),
            b';' => Some(Self::Semicolon),
            b'=' => Some(Self::Equal),
            b'+' => Some(Self::Plus),
            b'-' => Some(Self::Minus),
            b'*' => Some(Self::Asterisk),
            b'/' => Some(Self::Slash),
            b'&' => Some(Self::And),
            b'%' => Some(Self::Percent),
            b'|' => Some(Self::Pipe),
            b'\'' => Some(Self::Quote),
            b'"' => Some(Self::DoubleQuote),
            b'`' => Some(Self::Backtick),
            b'[' => Some(Self::BracketOpen),
            b']' => Some(Self::BracketClose),
            b'(' => Some(Self::ParenthesesOpen),
            b')' => Some(Self::ParenthesesClose),
            b'{' => Some(Self::CurlyBraceOpen),
            b'}' => Some(Self::CurlyBraceClose),
            b'<' => Some(Self::AngleBracketOpen),
            b'>' => Some(Self::AngleBracketClose),
            _ => None,
        }
    }

    pub(crate) fn is_separator(c: char) -> bool {
        Self::from_char(c).is_some()
    }

    pub(crate) fn is_separator_u8(c: u8) -> bool {
        Self::from_u8(c).is_some()
    }
}
