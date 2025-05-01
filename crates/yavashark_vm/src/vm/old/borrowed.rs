mod storage;

use crate::execute_old::Execute;
use crate::{Registers, Stack, VM};
use std::path::PathBuf;
use yavashark_bytecode::control::{ControlBlock, TryBlock};
use yavashark_bytecode::data::{ControlIdx, DataSection, OutputData, OutputDataType};
use yavashark_bytecode::{ConstIdx, Instruction, Reg, VarName};
use yavashark_env::error::ErrorObj;
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

    continue_storage: Option<OutputDataType>,

    try_stack: Vec<TryBlock>,

    throw: Option<Error>,
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
            continue_storage: None,
            try_stack: Vec::new(),
            throw: None,
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
            continue_storage: None,
            try_stack: Vec::new(),
            throw: None,
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

    pub fn handle_error(&mut self, err: Error) -> Res {
        if let Some(tb) = self.try_stack.last_mut() {
            if let Some(catch) = tb.catch.take() {
                if tb.finally.is_none() {
                    self.try_stack.pop();
                }
                self.offset_pc(catch);
                self.set_acc(ErrorObj::error_to_value(err, &self.realm));
            } else if let Some(finally) = tb.finally.take() {
                self.throw = Some(err);
                self.offset_pc(finally);

                self.try_stack.pop();
            }
        }

        Ok(())
    }
}

impl VM for OldBorrowedVM<'_> {
    fn acc(&self) -> Value {
        self.acc()
    }

    fn set_acc(&mut self, value: Value) {
        self.set_acc(value);
    }

    fn get_variable(&self, name: VarName) -> yavashark_env::Res<Value> {
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

    fn get_realm_ref(&self) -> &Realm {
        self.realm
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
        self.continue_storage = Some(out.data_type());
    }

    fn enter_try(&mut self, id: ControlIdx) -> Res {
        let Some(c) = self.data.control.get(id.0 as usize) else {
            return Err(Error::new("Invalid control index"));
        };

        let ControlBlock::Try(tb) = c else {
            return Err(Error::new("Control block is not a try block"));
        };

        self.try_stack.push(*tb);

        Ok(())
    }

    fn leave_try(&mut self) -> Res {
        let tb = self
            .try_stack
            .last_mut()
            .ok_or(Error::new("No try block"))?;

        if let Some(f) = tb.finally.take() {
            self.offset_pc(f);
        } else {
            let exit = tb.exit;

            if let Some(err) = self.throw.take() {
                return self.handle_error(err);
            }

            self.offset_pc(exit);
            self.try_stack.pop();
        }

        Ok(())
    }
}
