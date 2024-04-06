pub mod ast;
mod block;
mod function;
mod types;

use yavashark_lexer::tokens::Token;

struct Parser {
    tokens: Vec<Token>,
}
