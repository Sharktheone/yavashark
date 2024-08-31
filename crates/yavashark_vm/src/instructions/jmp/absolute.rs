use yavashark_bytecode::{JmpAddr, VarName};
use yavashark_env::{Value, Variable};
use crate::VM;

pub fn jmp(target: JmpAddr, vm: &mut VM) {
    vm.set_pc(target);
}



pub fn jmp_if(target: JmpAddr, name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    if value.is_truthy() {
        vm.set_pc(target);
    }
}

pub fn jmp_if_acc(target: JmpAddr, vm: &mut VM) {
    let value = vm.acc();
    if value.is_truthy() {
        vm.set_pc(target);
    }
}

pub fn jmp_if_not(target: JmpAddr, name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    if !value.is_truthy() {
        vm.set_pc(target);
    }
}

pub fn jmp_if_not_acc(target: JmpAddr, vm: &mut VM) {
    let value = vm.acc();
    if !value.is_truthy() {
        vm.set_pc(target);
    }
}

pub fn jmp_null(target: JmpAddr, name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    if value == Value::Null {
        vm.set_pc(target);
    }
}

pub fn jmp_null_acc(target: JmpAddr, vm: &mut VM) {
    let value = vm.acc();
    if value == Value::Null {
        vm.set_pc(target);
    }
}


pub fn jmp_undef(target: JmpAddr, name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    if value == Value::Undefined {
        vm.set_pc(target);
    }
}

pub fn jmp_undef_acc(target: JmpAddr, vm: &mut VM) {
    let value = vm.acc();
    if value == Value::Undefined {
        vm.set_pc(target);
    }
}

pub fn jmp_null_undef(target: JmpAddr, name: VarName, vm: &mut VM) {
    let value = vm.get_variable(name);
    if value == Value::Null || value == Value::Undefined {
        vm.set_pc(target);
    }
}

pub fn jmp_null_undef_acc(target: JmpAddr, vm: &mut VM) {
    let value = vm.acc();
    if value == Value::Null || value == Value::Undefined {
        vm.set_pc(target);
    }
}
