mod async_vm;
mod borrowed;
mod old;
mod owned;

pub use async_vm::*;
pub use borrowed::*;
pub use old::borrowed::*;
pub use old::owned::*;
pub use owned::*;

use yavashark_bytecode::data::{ControlIdx, Label, OutputData};
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::scope::Scope;
use yavashark_env::{Res, Value};

pub trait VM {
    fn acc(&self) -> Value;
    fn set_acc(&mut self, value: Value);
    fn get_variable(&mut self, name: VarName) -> Res<Value>;
    fn var_name(&self, name: VarName) -> Option<&str>;
    fn get_register(&self, reg: Reg) -> Res<Value>;
    fn get_label(&self, label: Label) -> Res<&str>;
    fn set_variable(&mut self, name: VarName, value: Value) -> Res;
    fn set_register(&mut self, reg: Reg, value: Value) -> Res;
    fn push(&mut self, value: Value);
    fn pop(&mut self) -> Option<Value>;
    fn set_accb(&mut self, value: bool);
    fn get_this(&self) -> yavashark_env::Res<Value>;
    fn get_constant(&self, const_idx: ConstIdx) -> yavashark_env::Res<Value>;
    #[must_use]
    fn get_stack(&self, idx: u32) -> Option<Value>;
    fn set_stack(&mut self, idx: u32, value: Value) -> Res;
    fn get_args(&mut self, num: u16) -> Vec<Value>;

    fn get_realm(&mut self) -> &mut yavashark_env::Realm;

    fn set_pc(&mut self, pc: usize);

    fn offset_pc(&mut self, offset: isize);

    fn push_scope(&mut self) -> Res;

    fn pop_scope(&mut self) -> Res;

    fn push_call_args(&mut self, args: Vec<Value>);

    fn push_call_arg(&mut self, arg: Value);

    fn get_call_args(&mut self) -> Vec<Value>;

    fn get_scope(&self) -> &Scope;

    fn get_scope_mut(&mut self) -> &mut Scope;

    fn set_continue_storage(&mut self, out: impl OutputData);

    fn enter_try(&mut self, id: ControlIdx) -> Res;
    fn leave_try(&mut self) -> Res;
}
