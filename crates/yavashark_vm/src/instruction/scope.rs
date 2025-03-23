use yavashark_env::Res;
use crate::VM;

pub fn push_scope(vm: &mut impl VM) -> Res {
    vm.push_scope()
}

pub fn pop_scope(vm: &mut impl VM) -> Res {
    vm.pop_scope()
}