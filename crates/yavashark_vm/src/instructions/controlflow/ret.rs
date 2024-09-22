use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::value::Value;
use yavashark_env::{ControlFlow, ControlResult};

pub fn return_() -> ControlResult {
    Err(ControlFlow::Return(Value::Undefined))
}

pub fn return_acc(vm: &mut VM) -> ControlResult {
    let value = vm.acc();
    Err(ControlFlow::Return(value))
}

pub fn return_reg(reg: Reg, vm: &mut VM) -> ControlResult {
    let value = vm.get_register(reg)?;
    Err(ControlFlow::Return(value))
}

pub fn return_var(var: VarName, vm: &mut VM) -> ControlResult {
    let value = vm.get_variable(var)?;
    Err(ControlFlow::Return(value))
}
