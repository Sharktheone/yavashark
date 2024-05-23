use std::any::Any;

use swc_ecma_ast::Stmt;

use yavashark_value::{Ctx, Obj};

use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;

mod prototypes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub(crate) proto: prototypes::Prototypes,
}

impl Context {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            proto: prototypes::Prototypes::new()?,
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
