pub mod ast;

use yavashark_lexer::tokens::Token;

struct Parser {
    tokens: Vec<Token>,
}

