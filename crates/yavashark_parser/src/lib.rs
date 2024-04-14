pub mod ast;
mod block;
mod declaration;
mod function;
mod statement;
mod types;
mod expression;
mod identifier;

use yavashark_lexer::tokens::Token;

pub struct Parser {
    pub tokens: Vec<Token>,
}
