use crate::Validator;
use swc_ecma_ast::DebuggerStmt;

impl Validator<'_> {
    pub const fn validate_debugger(&mut self, _debugger: &DebuggerStmt) -> Result<(), String> {
        Ok(())
    }
}
