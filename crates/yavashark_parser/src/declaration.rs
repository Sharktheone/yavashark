//! 14 ECMAScript Language; Statements and *Declarations*

use crate::declaration::function::{AsyncFunctionDeclaration, FunctionDeclaration};
use crate::declaration::generator::{AsyncGeneratorDeclaration, GeneratorDeclaration};

mod function;
mod generator;

pub enum Declaration {
    // Yield, Await
    Hoistable(HoistableDecl), // ?Yield, ?Await, ~Default
    Class(ClassDecl),         // ?Yield, ?Await, ~Default
    Lexical(LexicalDecl),     // +In ?Yield, ?Await
}

pub struct HoistableDecl {
    declaration: HoistableDeclaration,
}

pub struct ClassDecl {
    // declaration: ClassDeclaration,
}

pub struct LexicalDecl {
    // declaration: LexicalDeclaration,
}

pub enum HoistableDeclaration {
    // Yield, Await, Default
    Function(FunctionDeclaration),           // ?Yield, ?Await, ?Default
    Generator(GeneratorDeclaration),         // ?Yield, ?Await, ?Default
    AsyncFunction(AsyncFunctionDeclaration), // ?Yield, ?Await, ?Default
    AsyncGenerator(AsyncGeneratorDeclaration), // ?Yield, ?Await, ?Default
}
