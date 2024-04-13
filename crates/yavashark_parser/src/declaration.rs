//! 14 ECMAScript Language; Statements and *Declarations*

pub enum Declaration {
    // Yield, Await
    HoistableDecl(HoistableDecl), // ?Yield, ?Await, ~Default
    ClassDecl(ClassDecl),         // ?Yield, ?Await, ~Default
    LexicalDecl(LexicalDecl),     // +In ?Yield, ?Await
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
    FunctionDeclaration(FunctionDeclaration), // ?Yield, ?Await, ?Default
    GeneratorDeclaration(GeneratorDeclaration), // ?Yield, ?Await, ?Default
    AsyncFunctionDeclaration(AsyncFunctionDeclaration), // ?Yield, ?Await, ?Default
    AsyncGeneratorDeclaration(AsyncGeneratorDeclaration), // ?Yield, ?Await, ?Default
}
