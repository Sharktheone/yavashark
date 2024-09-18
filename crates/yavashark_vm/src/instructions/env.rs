use yavashark_bytecode::{Reg, VarName};
use crate::VM;

pub fn load_env(name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    vm.set_acc(value);
}

pub fn load_env_reg(name: VarName, reg: Reg, vm: &mut VM) {
    let value = vm.get_variable(name);
    
    vm.set_register(reg, value);
}
