use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_block(&mut self, block: &swc_ecma_ast::BlockStmt) -> Res {
        self.compile_statements(&block.stmts)
    }
}