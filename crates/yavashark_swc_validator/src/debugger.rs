use crate::Validator;
use swc_ecma_ast::DebuggerStmt;

impl Validator {
    pub fn validate_debugger(_debugger: &DebuggerStmt) -> Result<(), String> {
        Ok(())
    }
}
