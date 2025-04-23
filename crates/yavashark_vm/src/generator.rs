use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use yavashark_bytecode::BytecodeFunctionCode;
use yavashark_env::{Realm, ValueResult, Value, MutObject, Object, ObjectHandle, Res};
use yavashark_env::scope::Scope;
use yavashark_macro::{object, props};
use yavashark_value::{Error, Func, Obj};
use crate::{GeneratorPoll, ResumableVM, VmState};

#[object(function)]
#[derive(Debug)]
pub struct GeneratorFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
}

impl GeneratorFunction {
    #[must_use] 
    pub fn new(code: Rc<BytecodeFunctionCode>, scope: Scope, realm: &Realm) -> Self {
        Self { 
            inner: RefCell::new(MutableGeneratorFunction {
                object: MutObject::with_proto(realm.intrinsics.generator_function.clone().into())
            }),
            code,
            scope
        }
    }
}

#[props]
impl GeneratorFunction {
    
}

impl Func<Realm> for GeneratorFunction {
    fn call(&self, realm: &mut Realm, _args: Vec<Value>, _this: Value) -> ValueResult {
        let generator = Generator::new(realm, Rc::clone(&self.code), self.scope.clone());
        
        Ok(generator.into_value())
    }
}


#[object]
pub struct Generator {
    state: RefCell<Option<VmState>>,
}

impl Generator {
    pub fn new(realm: &Realm, code: Rc<BytecodeFunctionCode>, scope: Scope) -> Self {
        let state = VmState::new(code, scope);
        Self { 
            inner: RefCell::new(MutableGenerator {
                object: MutObject::with_proto(realm.intrinsics.generator.clone().into()),
            }),
            state: RefCell::new(Some(state)),
        }
    }
    
    pub fn init(realm: &mut Realm) -> Res {
        let gf = GeneratorFunction::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone().into()),
            realm.intrinsics.func.clone().into()
        )?;
        
        let g = Generator::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone().into()),
            realm.intrinsics.func.clone().into()
        )?;
        
        realm.intrinsics.generator_function = gf;
        realm.intrinsics.generator = g;
        
        Ok(())
    }
}

#[props]
impl Generator {
    pub fn next(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let Some(state) = self.state.take() else {
            let obj = Object::new(realm);
            
            obj.define_property("done".into(), true.into())?;
            obj.define_property("value".into(), Value::Undefined)?;
            
            return Ok(obj)
        };
        
        let vm = ResumableVM::from_state(state, realm);
        
        match vm.next() {
            GeneratorPoll::Yield(state, val) => {
                self.state.replace(Some(state));
                
                let obj = Object::new(realm);
                
                obj.define_property("done".into(), false.into())?;
                obj.define_property("value".into(), val)?;
                
                Ok(obj)
                
            }
            GeneratorPoll::Ret(res) => {
                let val = res?;
                
                let obj = Object::new(realm);
                
                obj.define_property("done".into(), true.into())?;
                obj.define_property("value".into(), val)?;
                
                Ok(obj)
            }
        }
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generator")
            .finish()
    }
}