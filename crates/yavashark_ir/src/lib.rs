use yavashark_string::YSString;

/// This IR will be used for many optimizations in the YS interpreter and compiler.
pub struct YavasharkIr<'alloc> {
    functions: bumpalo::collections::Vec<'alloc, Function<'alloc>>,

}



pub struct Function<'alloc> {
    name: YSString,
    parameters: Vec<YSString>,
    body: Statement<'alloc>,
}




pub struct YavasharkBlock<'alloc> {
    statements: bumpalo::collections::Vec<'alloc, YavasharkStatement<'alloc>>,
}


trait IRTranslator {
    fn translate(&self, input: &str) -> YavasharkIr;
}
