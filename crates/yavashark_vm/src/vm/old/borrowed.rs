mod storage;

use crate::execute_old::Execute;
use crate::{Registers, Stack, VM};
use std::path::PathBuf;
use yavashark_bytecode::data::{DataSection, OutputData};
use yavashark_bytecode::{ConstIdx, Instruction, Reg, VarName};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value, ValueResult};

pub struct OldBorrowedVM<'a> {
    regs: Registers,
    stack: Stack,
    call_args: Vec<Value>,

    pc: usize,
    code: &'a [Instruction],
    data: &'a DataSection,

    current_scope: Scope,

    acc: Value,

    realm: &'a mut Realm,
}

impl<'a> OldBorrowedVM<'a> {
    pub fn new(
        code: &'a [Instruction],
        data: &'a DataSection,
        realm: &'a mut Realm,
        file: PathBuf,
    ) -> Res<Self> {
        Ok(Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(realm, file),
            acc: Value::Undefined,
            realm,
        })
    }

    #[must_use]
    pub const fn with_scope(
        code: &'a [Instruction],
        data: &'a DataSection,
        realm: &'a mut Realm,
        scope: Scope,
    ) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            call_args: Vec::new(),
            pc: 0,
            code,
            data,
            current_scope: scope,
            acc: Value::Undefined,
            realm,
        }
    }
    pub fn get_realm(&mut self) -> &mut Realm {
        self.realm
    }

    pub fn push_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.child()?;

        Ok(())
    }

    pub fn pop_scope(&mut self) -> Res {
        let scope = self.current_scope.parent()?;

        if let Some(p) = scope {
            self.current_scope = p.into();
        } else {
            return Err(Error::new("No parent scope"));
        }

        Ok(())
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.pc = (self.pc as isize + offset) as usize;
    }

    pub fn run(&mut self) -> Res {
        while self.pc < self.code.len() {
            let instr = self.code[self.pc];
            self.pc += 1;

            instr.execute(self)?;
        }

        Ok(())
    }

    pub fn run_ret(&mut self) -> ValueResult {
        self.run()?;

        Ok(self.acc())
    }
}

impl VM for OldBorrowedVM<'_> {
    fn acc(&self) -> Value {
        self.acc()
    }

    fn set_acc(&mut self, value: Value) {
        self.set_acc(value);
    }

    fn get_variable(&mut self, name: VarName) -> yavashark_env::Res<Value> {
        self.get_variable(name)
    }

    fn var_name(&self, name: VarName) -> Option<&str> {
        self.var_name(name)
    }

    fn get_register(&self, reg: Reg) -> Res<Value> {
        self.get_register(reg)
    }
    
    fn get_label(&self, label: yavashark_bytecode::data::Label) -> Res<&str> {
        self.get_label(label)
    }

    fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        self.set_variable(name, value)
    }

    fn set_register(&mut self, reg: Reg, value: Value) -> Res {
        self.set_register(reg, value)
    }

    fn push(&mut self, value: Value) {
        self.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.pop()
    }

    fn set_accb(&mut self, value: bool) {
        self.set_accb(value);
    }

    fn get_this(&self) -> yavashark_env::Res<Value> {
        self.get_this()
    }

    fn get_constant(&self, const_idx: ConstIdx) -> yavashark_env::Res<Value> {
        self.get_constant(const_idx)
    }

    fn get_stack(&self, idx: u32) -> Option<Value> {
        self.get_stack(idx)
    }

    fn set_stack(&mut self, idx: u32, value: Value) -> Res {
        self.set_stack(idx, value)
    }

    fn get_args(&mut self, num: u16) -> Vec<Value> {
        self.get_args(num)
    }

    fn get_realm(&mut self) -> &mut yavashark_env::Realm {
        self.get_realm()
    }

    fn set_pc(&mut self, pc: usize) {
        self.set_pc(pc);
    }

    fn offset_pc(&mut self, offset: isize) {
        self.offset_pc(offset);
    }

    fn push_scope(&mut self) -> Res {
        self.push_scope()
    }

    fn pop_scope(&mut self) -> Res {
        self.pop_scope()
    }

    fn push_call_args(&mut self, args: Vec<Value>) {
        self.call_args.extend(args);
    }

    fn push_call_arg(&mut self, arg: Value) {
        self.call_args.push(arg);
    }

    fn get_call_args(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.call_args)
    }

    fn get_scope(&self) -> &Scope {
        &self.current_scope
    }
    
    fn get_scope_mut(&mut self) -> &mut Scope {
        &mut self.current_scope
    }

    fn set_continue_storage(&mut self, out: impl OutputData) {
        todo!()
    }
}
