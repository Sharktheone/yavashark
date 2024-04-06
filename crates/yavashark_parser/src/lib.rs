pub mod ast;
mod block;
mod function;
mod types;

use yavashark_lexer::tokens::Token;

pub struct Parser {
    pub tokens: Vec<Token>,
}
