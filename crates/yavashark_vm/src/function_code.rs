use crate::OldBorrowedVM;
use std::any::Any;
use yavashark_bytecode::data::DataSection;
use yavashark_bytecode::Instruction;
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

#[derive(Debug)]
pub struct BytecodeFunction {
    pub instructions: Vec<Instruction>,
    pub ds: DataSection,
}

impl FunctionCode for BytecodeFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult {
        let scope = Scope::with_parent_this(scope, this)?;

        println!("Running bytecode with VM!");

        let mut vm = OldBorrowedVM::with_scope(&self.instructions, &self.ds, realm, scope);

        vm.run()?;

        Ok(vm.acc())
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}
