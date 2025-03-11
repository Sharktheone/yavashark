use swc_ecma_ast::Stmt;

pub use yavashark_bytecode as bytecode;
pub use yavashark_codegen as codegen;
pub use yavashark_vm as vm;

use yavashark_codegen::ByteCodegen;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, ValueResult};
use yavashark_vm::BorrowedVM;
use yavashark_vm::yavashark_bytecode::data::DataSection;

pub struct ByteCodeInterpreter;

impl ByteCodeInterpreter {
    pub fn run_in(script: &Vec<Stmt>, realm: &mut Realm, scope: &mut Scope) -> ValueResult {
        let code = ByteCodegen::compile(script)
            .map_err(|e| Error::new_error(format!("Failed to compile: {e:?}")))?;

        let ds = DataSection::new(code.variables, code.literals);

        let mut vm = BorrowedVM::with_scope(&code.instructions, &ds, realm, scope.clone());

        vm.run_ret()
    }
}
