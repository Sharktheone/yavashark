use std::any::{type_name, type_name_of_val, Any, TypeId};
use std::cell::RefMut;
use anyhow::anyhow;

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

impl Context {
    pub fn new() -> Result<Self, anyhow::Error> {
        let obj_prototype: Box<dyn Obj<Self>> = Box::new(Prototype::new());

        let obj_prototype = ObjectHandle::new(obj_prototype);

        let func_prototype: Box<dyn Obj<Self>> =
            Box::new(FunctionPrototype::new(&obj_prototype.clone().into()));
        let func_prototype = ObjectHandle::new(func_prototype);

        {
            let mut obj: RefMut<Box<dyn Obj<Self>>> = obj_prototype.get_mut().map_err(|e| anyhow!(format!("{e:?}")))?;
            let obj = obj.as_any_mut();

            let proto = obj
                .downcast_mut::<Prototype>()
                .ok_or_else(|| anyhow!("downcast_mut::<Prototype> failed"))?;

            proto.initialize(func_prototype.clone().into());
        }

        Ok(Self {
            obj_prototype,
            func_prototype,
        })
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
