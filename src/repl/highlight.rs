use std::ops::Range;

const KEYWORDS: &[&str] = &[
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "of",
    "return",
    "static",
    "switch",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

const CONSTANTS: &[&str] = &[
    "true",
    "false",
    "null",
    "undefined",
    "this",
    "super",
    "NaN",
    "Infinity",
];

const RESET: &str = "\x1b[0m";
const MATCHING_BRACKET: &str = "\x1b[1;34m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Whitespace,
    Comment,
    Keyword,
    Constant,
    Ident,
    Number,
    String,
    Template,
    Regex,
    Punct,
}

impl TokenKind {
    const fn color(self) -> Option<&'static str> {
        match self {
            Self::Keyword => Some("\x1b[35m"),
            Self::Constant | Self::Number => Some("\x1b[33m"),
            Self::String | Self::Template => Some("\x1b[32m"),
            Self::Regex => Some("\x1b[31m"),
            Self::Comment => Some("\x1b[90m"),
            Self::Whitespace | Self::Ident | Self::Punct => None,
        }
    }

    const fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    range: Range<usize>,
}

pub fn highlight(line: &str, cursor: usize) -> String {
    let tokens = Lexer::new(line).tokenize();
    let brackets = matching_brackets(line, &tokens, cursor);

    let mut out = String::with_capacity(line.len() * 2);

    for (i, token) in tokens.iter().enumerate() {
        let text = &line[token.range.clone()];

        let color = if brackets.is_some_and(|(a, b)| i == a || i == b) {
            Some(MATCHING_BRACKET)
        } else {
            token.kind.color()
        };

        match color {
            Some(color) => {
                out.push_str(color);
                out.push_str(text);
                out.push_str(RESET);
            }
            None => out.push_str(text),
        }
    }

    out
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    template_braces: Vec<usize>,
    last_significant: Option<Token>,
}

impl<'a> Lexer<'a> {
    const fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            template_braces: Vec::new(),
            last_significant: None,
        }
    }

    fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            if !token.kind.is_trivia() {
                self.last_significant = Some(token.clone());
            }

            tokens.push(token);
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let start = self.pos;
        let c = self.peek()?;

        let kind = match c {
            c if c.is_whitespace() => {
                self.eat_while(char::is_whitespace);
                TokenKind::Whitespace
            }
            '/' if self.rest().starts_with("//") => {
                self.eat_while(|c| c != '\n');
                TokenKind::Comment
            }
            '/' if self.rest().starts_with("/*") => {
                self.block_comment();
                TokenKind::Comment
            }
            '/' if self.regex_allowed() => {
                self.regex();
                TokenKind::Regex
            }
            '"' | '\'' => {
                self.string(c);
                TokenKind::String
            }
            '`' => {
                self.bump();
                self.template();
                TokenKind::Template
            }
            '}' if self.template_braces.last() == Some(&0) => {
                self.template_braces.pop();
                self.bump();
                self.template();
                TokenKind::Template
            }
            c if c.is_ascii_digit()
                || (c == '.' && self.peek_nth(1).is_some_and(|c| c.is_ascii_digit())) =>
            {
                self.eat_while(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
                TokenKind::Number
            }
            c if is_ident_start(c) => {
                self.eat_while(is_ident_part);
                self.classify_word(&self.src[start..self.pos])
            }
            _ => {
                self.punct(c);
                TokenKind::Punct
            }
        };

        Some(Token {
            kind,
            range: start..self.pos,
        })
    }

    fn classify_word(&self, word: &str) -> TokenKind {
        if self.last_significant_text() == Some(".") {
            TokenKind::Ident
        } else if KEYWORDS.contains(&word) {
            TokenKind::Keyword
        } else if CONSTANTS.contains(&word) {
            TokenKind::Constant
        } else {
            TokenKind::Ident
        }
    }

    fn regex_allowed(&self) -> bool {
        let Some(last) = &self.last_significant else {
            return true;
        };

        match last.kind {
            TokenKind::Keyword => true,
            TokenKind::Punct => !matches!(self.last_significant_text(), Some(")" | "]" | "}")),
            _ => false,
        }
    }

    fn punct(&mut self, c: char) {
        self.bump();

        if let Some(depth) = self.template_braces.last_mut() {
            match c {
                '{' => *depth += 1,
                '}' => *depth -= 1,
                _ => {}
            }
        }
    }

    fn string(&mut self, quote: char) {
        self.bump();

        while let Some(c) = self.bump() {
            match c {
                '\\' => _ = self.bump(),
                '\n' => break,
                c if c == quote => break,
                _ => {}
            }
        }
    }

    fn template(&mut self) {
        while let Some(c) = self.bump() {
            match c {
                '\\' => _ = self.bump(),
                '`' => break,
                '$' if self.peek() == Some('{') => {
                    self.bump();
                    self.template_braces.push(0);
                    break;
                }
                _ => {}
            }
        }
    }

    fn regex(&mut self) {
        self.bump();
        let mut in_class = false;

        while let Some(c) = self.bump() {
            match c {
                '\\' => _ = self.bump(),
                '[' => in_class = true,
                ']' => in_class = false,
                '/' if !in_class => break,
                '\n' => break,
                _ => {}
            }
        }

        self.eat_while(is_ident_part);
    }

    fn block_comment(&mut self) {
        match self.rest()[2..].find("*/") {
            Some(end) => self.pos += end + 4,
            None => self.pos = self.src.len(),
        }
    }

    fn last_significant_text(&self) -> Option<&'a str> {
        self.last_significant
            .as_ref()
            .map(|token| &self.src[token.range.clone()])
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.rest().chars().nth(n)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn eat_while(&mut self, pred: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&pred) {
            self.bump();
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphabetic()
}

fn is_ident_part(c: char) -> bool {
    is_ident_start(c) || c.is_alphanumeric()
}

fn matching_brackets(src: &str, tokens: &[Token], cursor: usize) -> Option<(usize, usize)> {
    let bracket_at = |pos: usize| {
        tokens.iter().position(|token| {
            token.kind == TokenKind::Punct
                && token.range.start == pos
                && bracket_pair(&src[token.range.clone()]).is_some()
        })
    };

    let current = cursor
        .checked_sub(1)
        .and_then(bracket_at)
        .or_else(|| bracket_at(cursor))?;

    let (open, close, forward) = bracket_pair(&src[tokens[current].range.clone()])?;


    let tmp1;
    let tmp2;

    let candidates: &dyn Iterator<Item = usize> = if forward {
        tmp1 = current + 1..tokens.len();

        &tmp1
    } else {
        tmp2 = (0..current).rev();
        &tmp2
    };

    let mut depth = 0usize;

    for i in candidates {
        let token = &tokens[i];
        if token.kind != TokenKind::Punct {
            continue;
        }

        let text = &src[token.range.clone()];

        if text == open {
            depth += 1;
        } else if text == close {
            if depth == 0 {
                return Some((current, i));
            }
            depth -= 1;
        }
    }

    None
}

fn bracket_pair(text: &str) -> Option<(&'static str, &'static str, bool)> {
    Some(match text {
        "(" => ("(", ")", true),
        "[" => ("[", "]", true),
        "{" => ("{", "}", true),
        ")" => (")", "(", false),
        "]" => ("]", "[", false),
        "}" => ("}", "{", false),
        _ => return None,
    })
}