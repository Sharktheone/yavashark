use std::any::Any;
use std::path::{Path, PathBuf};
use log::info;
use yavashark_bytecode::{Bytecode, Instruction};
use yavashark_bytecode::data::DataSection;
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::{Realm, RuntimeResult, Value};
use yavashark_env::scope::Scope;
use crate::{BorrowedVM, VM};

#[derive(Debug)]
pub struct BytecodeFunction {
    pub instructions: Vec<Instruction>,
    pub ds: DataSection,
}

impl FunctionCode for BytecodeFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult {
        let scope = Scope::with_parent_this(scope, this)?;

        println!("Running bytecode with VM!");

        let mut vm = BorrowedVM::with_scope(&self.instructions, &self.ds, realm, scope);

        vm.run()?;

        Ok(vm.acc())
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}