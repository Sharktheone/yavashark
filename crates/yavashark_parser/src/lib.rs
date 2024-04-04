pub mod ast;
mod function;
mod types;
mod block;

use yavashark_lexer::tokens::Token;

struct Parser {
    tokens: Vec<Token>,
}

