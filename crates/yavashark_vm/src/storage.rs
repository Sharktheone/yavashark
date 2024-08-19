use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Value;

impl VM {
    pub fn get_variable(&self, name: VarName) -> Value {
        todo!()
    }

    pub fn get_register(&self, reg: Reg) -> Value {
        todo!()
    }

    pub fn set_variable(&mut self, name: VarName, value: Value) {
        todo!()
    }

    pub fn set_register(&mut self, reg: Reg, value: Value) {
        todo!()
    }

    pub fn push(&mut self, value: Value) {
        todo!()
    }

    pub fn pop(&mut self) -> Value {
        todo!()
    }

    pub fn acc(&self) -> Value {
        todo!()
    }

    pub fn set_acc(&mut self, value: Value) {
        todo!()
    }

    pub fn set_accb(&mut self, value: bool) {
        todo!()
    }
}
