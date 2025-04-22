use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use yavashark_bytecode::BytecodeFunctionCode;
use yavashark_env::{Realm, ValueResult, Value, MutObject};
use yavashark_env::scope::Scope;
use yavashark_macro::object;
use yavashark_value::{Error, Func, Obj};
use crate::VmState;

#[object(function)]
#[derive(Debug)]
pub struct GeneratorFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
}

impl Func<Realm> for GeneratorFunction {
    fn call(&self, realm: &mut Realm, _args: Vec<Value>, _this: Value) -> ValueResult {
        let generator = Generator::new(realm, Rc::clone(&self.code), self.scope.clone());
        
        Ok(generator.into_value())
    }
}

#[object]
pub struct Generator {
    state: Option<VmState>,
}

impl Generator {
    pub fn new(realm: &Realm, code: Rc<BytecodeFunctionCode>, scope: Scope) -> Self {
        let state = VmState::new(code, scope);
        Self { 
            inner: RefCell::new(MutableGenerator {
                object: MutObject::new(realm)
            }),
            state: Some(state)
        }
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generator")
            .finish()
    }
}