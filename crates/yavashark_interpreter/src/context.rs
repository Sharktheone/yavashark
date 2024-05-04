use std::any::{type_name, type_name_of_val, Any, TypeId};
use std::cell::RefMut;

use swc_ecma_ast::Stmt;

use yavashark_value::{Ctx, Obj};

use crate::object::Prototype;
use crate::scope::Scope;
use crate::Value;
use crate::{FunctionPrototype, ObjectHandle, RuntimeResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub(crate) obj_prototype: ObjectHandle,
    pub(crate) func_prototype: ObjectHandle,
}

impl Default for Context {
    fn default() -> Self {
        let obj_prototype: Box<dyn Obj<Context>> = Box::new(Prototype::new());

        let obj_prototype = ObjectHandle::new(obj_prototype);

        let func_prototype: Box<dyn Obj<Context>> =
            Box::new(FunctionPrototype::new(&obj_prototype.clone().into()));
        let func_prototype = ObjectHandle::new(func_prototype);

        {
            let mut obj: RefMut<Box<dyn Obj<Context>>> = obj_prototype.get_mut().unwrap();
            let obj = obj.as_any_mut();

            let proto = obj
                .downcast_mut::<Prototype>()
                .expect("Failed to get prototype");

            proto.initialize(func_prototype.clone().into())
        }

        Self {
            obj_prototype,
            func_prototype,
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run_statements(&mut self, script: &Vec<Stmt>, scope: &mut Scope) -> RuntimeResult {
        let mut last_value = Value::Undefined;
        for stmt in script {
            last_value = self.run_statement(stmt, scope)?;
        }

        Ok(last_value)
    }
}

impl Ctx for Context {}
