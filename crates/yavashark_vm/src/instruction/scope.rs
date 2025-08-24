use crate::VM;
use yavashark_env::Res;

pub fn push_scope(vm: &mut impl VM) -> Res {
    vm.push_scope()
}

pub fn push_loop_scope(vm: &mut impl VM) -> Res {
    vm.push_scope()?;
    vm.get_scope_mut().state_set_loop()
}

pub fn pop_scope(vm: &mut impl VM) -> Res {
    vm.pop_scope()
}
