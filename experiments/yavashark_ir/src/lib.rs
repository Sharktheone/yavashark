use crate::statement::Stmt;
use yavashark_string::YSString;
use crate::atom::Atom;

mod atom;
mod expr;
mod statement;
mod span;

pub use atom::*;
pub use expr::*;
pub use statement::*;
pub use span::*;



/// This IR will be used for many optimizations in the YS interpreter and compiler.
pub struct YavasharkIr<'alloc> {
    functions: &'alloc [Function<'alloc>],

    root: Stmt<'alloc>,
}

pub struct Function<'alloc> {
    name: YSString,
    parameters: &'alloc [YSString],
    body: Stmt<'alloc>,
}

pub struct YavasharkBlock<'alloc> {
    statements: &'alloc [Stmt<'alloc>],
}

trait IRTranslator {
    fn translate(&self, input: &str) -> YavasharkIr;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ident {
    pub name: Atom,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeId(pub u32);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FuncId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MethodKind {
    Method,
    Getter,
    Setter
}