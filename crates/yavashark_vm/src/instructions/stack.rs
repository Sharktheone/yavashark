use yavashark_bytecode::{ConstIdx, Reg};
use crate::VM;

pub fn push_const(const_idx: ConstIdx, vm: &mut VM) {
    let value = vm.get_constant(const_idx);
    vm.push(value);
}

pub fn push_reg(reg: Reg, vm: &mut VM) {
    let value = vm.get_register(reg);
    vm.push(value);
}

pub fn push_acc(vm: &mut VM) {
    let value = vm.get_acc();
    vm.push(value);
}

pub fn pop(vm: &mut VM) {
    vm.pop();
}


pub fn pop_n(n: u32, vm: &mut VM) {
    for _ in 0..n {
        vm.pop();
    }
}

pub fn pop_to_reg(reg: Reg, vm: &mut VM) {
    let value = vm.pop();
    vm.set_register(reg, value);
}

pub fn pop_to_acc(vm: &mut VM) {
    let value = vm.pop();
    vm.set_acc(value);
}



pub fn stack_to_reg(reg: Reg, vm: &mut VM) {
    let value = vm.pop();
    vm.set_register(reg, value);
}

pub fn stack_to_acc(vm: &mut VM) {
    let value = vm.pop();
    vm.set_acc(value);
}

pub fn stack_idx_to_reg(reg: Reg, idx: u32, vm: &mut VM) {
    let value = vm.get_stack(idx);
    vm.set_register(reg, value);
}

pub fn stack_idx_to_acc(idx: u32, vm: &mut VM) {
    let value = vm.get_stack(idx);
    vm.set_acc(value);
}