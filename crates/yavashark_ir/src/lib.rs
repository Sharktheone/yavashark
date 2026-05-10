use crate::statement::Statement;
use bumpalo::collections::Vec;
use yavashark_string::YSString;

mod statement;
mod expr;

/// This IR will be used for many optimizations in the YS interpreter and compiler.
pub struct YavasharkIr<'alloc> {
    functions: Vec<'alloc, Function<'alloc>>,

    root: Statement<'alloc>,
}

pub struct Function<'alloc> {
    name: YSString,
    parameters: Vec<'alloc, YSString>,
    body: Statement<'alloc>,
}

pub struct YavasharkBlock<'alloc> {
    statements: Vec<'alloc, Statement<'alloc>>,
}

trait IRTranslator {
    fn translate(&self, input: &str) -> YavasharkIr;
}
