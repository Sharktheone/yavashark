pub mod ast;
mod block;
mod function;
mod types;
mod statement;
mod declaration;

use yavashark_lexer::tokens::Token;

pub struct Parser {
    pub tokens: Vec<Token>,
}
