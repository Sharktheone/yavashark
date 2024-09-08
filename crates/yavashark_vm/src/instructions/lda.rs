use crate::VM;
use yavashark_bytecode::{ConstIdx, Reg, VarName};

pub fn lda(name: VarName, const_idx: ConstIdx, vm: &mut VM) {
    let value = vm.get_constant(const_idx);

    vm.set_variable(name, value);
}

pub fn lda_acc(const_idx: ConstIdx, vm: &mut VM) {
    let value = vm.get_constant(const_idx);

    vm.set_acc(value);
}

pub fn lda_reg(reg: Reg, const_idx: ConstIdx, vm: &mut VM) {
    let value = vm.get_constant(const_idx);

    vm.set_register(reg, value);
}
