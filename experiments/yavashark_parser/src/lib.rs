#![allow(dead_code)]
#![allow(unused)]

use yavashark_lexer::tokens::Token;

pub mod ast;
mod block;
mod declaration;
mod expression;
mod function;
mod identifier;
mod statement;
mod types;

pub struct Parser {
    pub tokens: Vec<Token>,
}
