use crate::VM;

pub fn push_scope(vm: &mut VM) {
    vm.push_scope();
}

pub fn pop_scope(vm: &mut VM) {
    vm.pop_scope();
}
