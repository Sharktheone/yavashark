use crate::VM;
use yavashark_bytecode::data::{Acc, ConstIdx, Reg, Stack, VarName};
use yavashark_env::{Error, Res, Value};

pub trait Data: Copy {
    fn get(self, vm: &mut impl VM) -> Res<Value>;
}

pub trait OutputData: Data {
    fn set(self, value: Value, vm: &mut impl VM) -> Res;
}

impl Data for Acc {
    fn get(self, vm: &mut impl VM) -> Res<Value> {
        Ok(vm.acc())
    }
}

impl OutputData for Acc {
    fn set(self, value: Value, vm: &mut impl VM) -> Res {
        vm.set_acc(value);

        Ok(())
    }
}

impl Data for Reg {
    fn get(self, vm: &mut impl VM) -> Res<Value> {
        vm.get_register(self.0)
    }
}

impl OutputData for Reg {
    fn set(self, value: Value, vm: &mut impl VM) -> Res {
        vm.set_register(self.0, value)
    }
}

impl Data for VarName {
    fn get(self, vm: &mut impl VM) -> Res<Value> {
        vm.get_variable(self.0)
    }
}

impl OutputData for VarName {
    fn set(self, value: Value, vm: &mut impl VM) -> Res {
        vm.set_variable(self.0, value)
    }
}

impl Data for ConstIdx {
    fn get(self, vm: &mut impl VM) -> Res<Value> {
        vm.get_constant(self.0)
    }
}

impl Data for Stack {
    fn get(self, vm: &mut impl VM) -> Res<Value> {
        vm.get_stack(self.0)
            .ok_or_else(|| Error::new("Invalid stack index"))
    }
}

impl OutputData for Stack {
    fn set(self, value: Value, vm: &mut impl VM) -> Res {
        vm.set_stack(self.0, value)
    }
}
