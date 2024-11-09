use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, NativeFunction, Object, ObjectHandle, Realm, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::Error;

pub fn print(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::new(
        "print",
        |args, _, _| {
            let Some(first) = args.first() else {
                return Err(Error::ty("expected at least one argument"));
            };

            println!("{first:?}");

            Ok(Value::Undefined)
        },
        realm,
    )
}

#[object(direct(abstract_module_source(AbstractModuleSource)))]
#[derive(Debug)]
pub struct Test262 {
    #[allow(unused)]
    realm: Option<Realm>,
}

impl Test262 {
    pub fn new(realm: &Realm) -> Self {
        Self {
            object: Object::raw(realm),
            abstract_module_source: Value::Undefined.into(),
            realm: None,
        }
    }

    #[allow(unused)]
    pub fn with_realm(realm: &Realm, new_realm: Realm) -> Self {
        Self {
            object: Object::raw(realm),
            abstract_module_source: Value::Undefined.into(),
            realm: Some(new_realm),
        }
    }
}

#[properties]
#[allow(clippy::needless_pass_by_value)]
impl Test262 {
    #[prop("createRealm")]
    fn create_realm(&self, _args: Vec<Value>, realm: &Realm) -> ValueResult {
        let new_realm = Realm::new().map_err(|e| Error::new_error(e.to_string()))?;

        let global = new_realm.global.clone();

        let this: Value = ObjectHandle::new(Self::with_realm(realm, new_realm)).into();

        global.define_property("$262".into(), this.copy())?;

        Ok(this)
    }

    #[prop("detachArrayBuffer")]
    fn detach_array_buffer(&mut self, _args: Vec<Value>, _realm: &mut Realm) -> ValueResult {
        Ok(Value::Undefined)
    }

    #[prop("evalScript")]
    fn eval_script(&mut self, args: Vec<Value>, realm: &mut Realm) -> ValueResult {
        let input = args.first().ok_or(Error::ty("expected one argument"))?;

        let Value::String(input) = input else {
            return Err(Error::ty("expected string"));
        };

        if input.is_empty() {
            return Ok(Value::Undefined);
        }

        let input = StringInput::new(input, BytePos(0), BytePos(input.len() as u32 - 1));

        let c = EsSyntax::default();

        let mut p = Parser::new(Syntax::Es(c), input, None);

        let script = p
            .parse_script()
            .map_err(|e| Error::syn_error(format!("{e:?}")))?;

        let realm = self.realm.as_mut().unwrap_or(realm);

        let mut scope = Scope::global(realm, PathBuf::new());

        yavashark_interpreter::Interpreter::run_statements(realm, &script.body, &mut scope).or_else(
            |e| match e {
                ControlFlow::Error(e) => Err(e),
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Undefined),
            },
        )

        //TODO: we should respect, what interpreter is currently running. Since the bytecode is highly experimental ride now, this is okay.
    }

    #[allow(clippy::unused_self)]
    fn gc(&self, _args: Vec<Value>, _realm: &Realm) -> ValueResult {
        // gc is always handled automatically when something goes out of scope. We don't need an extra function for that.

        Ok(Value::Undefined)
    }
}
