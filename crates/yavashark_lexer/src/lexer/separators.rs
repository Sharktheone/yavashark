use crate::tokens::punct::PunctKind;

//Separators are characters that are used to separate tokens in the source code. Such as keywords, identifiers, literals, etc.
pub(crate) enum Separators {
    Space,
    NewLine,
    Tab,
    Pipe,
    Quote,
    DoubleQuote,
    Backtick,
    Punct(PunctKind),
}

impl Separators {
    pub(crate) fn from_char(c: char) -> Option<Self> {
        match c {
            ' ' => Some(Self::Space),
            '\n' => Some(Self::NewLine),
            '\t' => Some(Self::Tab),
            '|' => Some(Self::Pipe),
            '\'' => Some(Self::Quote),
            '"' => Some(Self::DoubleQuote),
            '`' => Some(Self::Backtick),
            ',' => Some(Self::Punct(PunctKind::Comma)),
            '.' => Some(Self::Punct(PunctKind::Dot)),
            ':' => Some(Self::Punct(PunctKind::Colon)),
            '?' => Some(Self::Punct(PunctKind::QuestionMark)),
            '!' => Some(Self::Punct(PunctKind::ExclamationMark)),
            ';' => Some(Self::Punct(PunctKind::Semicolon)),
            '=' => Some(Self::Punct(PunctKind::Equal)),
            '+' => Some(Self::Punct(PunctKind::Plus)),
            '-' => Some(Self::Punct(PunctKind::Minus)),
            '*' => Some(Self::Punct(PunctKind::Asterisk)),
            '/' => Some(Self::Punct(PunctKind::Slash)),
            '&' => Some(Self::Punct(PunctKind::And)),
            '%' => Some(Self::Punct(PunctKind::Percent)),
            '[' => Some(Self::Punct(PunctKind::BracketOpen)),
            ']' => Some(Self::Punct(PunctKind::BracketClose)),
            '(' => Some(Self::Punct(PunctKind::ParenthesesOpen)),
            ')' => Some(Self::Punct(PunctKind::ParenthesesClose)),
            '{' => Some(Self::Punct(PunctKind::CurlyBraceOpen)),
            '}' => Some(Self::Punct(PunctKind::CurlyBraceClose)),
            '<' => Some(Self::Punct(PunctKind::AngleBracketOpen)),
            '>' => Some(Self::Punct(PunctKind::AngleBracketClose)),
            _ => None,
        }
    }

    pub(crate) fn from_u8(c: u8) -> Option<Self> {
        match c {
            b' ' => Some(Self::Space),
            b'\n' => Some(Self::NewLine),
            b'\t' => Some(Self::Tab),
            b'|' => Some(Self::Pipe),
            b'\'' => Some(Self::Quote),
            b'"' => Some(Self::DoubleQuote),
            b'`' => Some(Self::Backtick),
            b',' => Some(Self::Punct(PunctKind::Comma)),
            b'.' => Some(Self::Punct(PunctKind::Dot)),
            b':' => Some(Self::Punct(PunctKind::Colon)),
            b'?' => Some(Self::Punct(PunctKind::QuestionMark)),
            b'!' => Some(Self::Punct(PunctKind::ExclamationMark)),
            b';' => Some(Self::Punct(PunctKind::Semicolon)),
            b'=' => Some(Self::Punct(PunctKind::Equal)),
            b'+' => Some(Self::Punct(PunctKind::Plus)),
            b'-' => Some(Self::Punct(PunctKind::Minus)),
            b'*' => Some(Self::Punct(PunctKind::Asterisk)),
            b'/' => Some(Self::Punct(PunctKind::Slash)),
            b'&' => Some(Self::Punct(PunctKind::And)),
            b'%' => Some(Self::Punct(PunctKind::Percent)),
            b'[' => Some(Self::Punct(PunctKind::BracketOpen)),
            b']' => Some(Self::Punct(PunctKind::BracketClose)),
            b'(' => Some(Self::Punct(PunctKind::ParenthesesOpen)),
            b')' => Some(Self::Punct(PunctKind::ParenthesesClose)),
            b'{' => Some(Self::Punct(PunctKind::CurlyBraceOpen)),
            b'}' => Some(Self::Punct(PunctKind::CurlyBraceClose)),
            b'<' => Some(Self::Punct(PunctKind::AngleBracketOpen)),
            b'>' => Some(Self::Punct(PunctKind::AngleBracketClose)),
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
