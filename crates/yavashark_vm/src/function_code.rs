use crate::task::BytecodeAsyncTask;
use crate::{BorrowedVM, OldBorrowedVM, VM};
use std::any::Any;
use std::rc::Rc;
use yavashark_bytecode::data::DataSection;
use yavashark_bytecode::{BytecodeFunctionCode, Instruction};
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

#[derive(Debug)]
pub struct OldBytecodeFunction {
    pub instructions: Vec<Instruction>,
    pub ds: DataSection,
}

impl FunctionCode for OldBytecodeFunction {
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

#[derive(Debug)]
pub struct BytecodeFunction {
    pub code: Rc<BytecodeFunctionCode>,
    pub is_async: bool,
    pub is_generator: bool,
}

impl FunctionCode for BytecodeFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult {
        let scope = Scope::with_parent_this(scope, this)?;

        if self.is_async {
            return Ok(BytecodeAsyncTask::new(Rc::clone(&self.code), realm, scope)?.into());
        }

        let mut vm = BorrowedVM::with_scope(&self.code.instructions, &self.code.ds, realm, scope);

        vm.run()?;

        Ok(vm.acc())
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct BytecodeArrowFunction {
    pub code: Rc<BytecodeFunctionCode>,
    pub this: Value,
    pub is_async: bool,
    pub is_generator: bool,
}

impl FunctionCode for BytecodeArrowFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, _: Value) -> RuntimeResult {
        let scope = Scope::with_parent_this(scope, self.this.copy())?;

        if self.is_async {
            return Ok(BytecodeAsyncTask::new(Rc::clone(&self.code), realm, scope)?.into());
        }

        let mut vm = BorrowedVM::with_scope(&self.code.instructions, &self.code.ds, realm, scope);

        vm.run()?;

        Ok(vm.acc())
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}
