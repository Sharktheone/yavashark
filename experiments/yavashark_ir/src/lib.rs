use crate::statement::Stmt;
use yavashark_string::YSString;

mod atom;
mod expr;
mod statement;

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
