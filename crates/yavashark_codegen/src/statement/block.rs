use swc_ecma_ast::BlockStmt;

use crate::ByteCodegen;

impl ByteCodegen {
    pub fn compile_block(&mut self, stmt: &BlockStmt) {
        for stmt in &stmt.stmts {
            self.compile_statement(stmt);
        }
    }
}
