use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Value;
use crate::VM;



pub fn load_member(target: VarName, member: VarName, vm: &mut VM) {
    let value = vm.get_variable(target);
    let member = value.get_member(member, vm);
    vm.set_acc(member);
}

pub fn load_member_acc(member: VarName, vm: &mut VM) {
    let acc = vm.acc();
    let value = acc.get_member(member, vm);
    vm.set_acc(value);
}


pub fn load_member_reg(target: Reg, member: VarName, vm: &mut VM) {
    let value = vm.get_register(target);
    let member = value.get_member(member, vm);
    vm.set_acc(member);
}

