use std::cell::RefCell;
use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::builtins::array_buf::ArrayBuffer;
use yavashark_env::scope::Scope;
use yavashark_env::{
    ControlFlow, Error, MutObject, NativeFunction, ObjectHandle, ObjectProperty, Realm, Value,
    ValueResult,
};
use yavashark_interpreter::eval::InterpreterEval;
use yavashark_macro::{object, properties_new};
use yavashark_swc_validator::Validator;

pub fn print(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::new(
        "print",
        |args, _, realm| {
            let Some(first) = args.first() else {
                return Err(Error::ty("expected at least one argument"));
            };

            let arg = first.to_string(realm)?;

            println!("{arg}");

            Ok(Value::Undefined)
        },
        realm,
    )
}

#[object(direct(abstract_module_source(AbstractModuleSource)))]
#[derive(Debug)]
pub struct Test262 {
    #[allow(unused)]
    #[mutable]
    realm: Option<Realm>,
}

impl Test262 {
    pub fn new(realm: &mut Realm) -> Self {
        let mut this = Self {
            inner: RefCell::new(MutableTest262 {
                object: MutObject::new(realm),
                abstract_module_source: ObjectProperty::new(Value::Undefined),
                realm: None,
            }),
        };

        this.initialize(realm).unwrap();

        this
    }

    #[allow(unused)]
    pub fn with_realm(realm: &mut Realm, new_realm: Realm) -> Self {
        let mut this = Self {
            inner: RefCell::new(MutableTest262 {
                object: MutObject::new(realm),
                abstract_module_source: Value::Undefined.into(),
                realm: Some(new_realm),
            }),
        };

        this.initialize(realm).unwrap();

        this
    }
}

#[properties_new(raw)]
#[allow(clippy::needless_pass_by_value)]
impl Test262 {
    #[prop("createRealm")]
    fn create_realm(&self, #[realm] realm: &mut Realm) -> ValueResult {
        let mut new_realm = Realm::new().map_err(|e| Error::new_error(e.to_string()))?;

        new_realm.set_eval(InterpreterEval, false)?;
        yavashark_vm::init(&mut new_realm)?;

        let global = new_realm.global.clone();

        let this: Value = ObjectHandle::new(Self::with_realm(realm, new_realm)).into();

        global.define_property("$262".into(), this.copy(), realm)?;

        this.define_property("global", global.into(), realm)?;

        Ok(this)
    }

    #[prop("detachArrayBuffer")]
    fn detach_array_buffer(buf: &ArrayBuffer) -> ValueResult {
        let _ = buf.detach();

        Ok(Value::Undefined)
    }

    #[prop("evalScript")]
    fn eval_script(&self, args: Vec<Value>, #[realm] realm: &mut Realm) -> ValueResult {
        let input = args.first().ok_or(Error::ty("expected one argument"))?;

        let Value::String(input) = input else {
            return Err(Error::ty("expected string"));
        };

        if input.is_empty() {
            return Ok(Value::Undefined);
        }

        let input = StringInput::new(input, BytePos(0), BytePos(input.len() as u32));

        let c = EsSyntax {
            jsx: false,
            fn_bind: false,
            decorators: true,
            decorators_before_export: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: false,
            allow_return_outside_function: false,
            auto_accessors: true,
            explicit_resource_management: true,
        };

        let mut p = Parser::new(Syntax::Es(c), input, None);

        let script = p
            .parse_script()
            .map_err(|e| Error::syn_error(format!("{e:?}")))?;

        let errors = p.take_errors();

        if !errors.is_empty() {
            return Err(Error::syn_error(format!("Parse errors: {errors:?}")));
        }

        if let Err(e) = Validator::new().validate_statements(&script.body) {
            return Err(Error::syn_error(e));
        }

        let mut inner = self.inner.try_borrow_mut()?;

        let realm = inner.realm.as_mut().unwrap_or(realm);

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
    fn gc() -> ValueResult {
        // gc is always handled automatically when something goes out of scope. We don't need an extra function for that.

        Ok(Value::Undefined)
    }

    #[prop("IsHTMLDDA")]
    fn is_html_dda() -> ValueResult {
        Ok(Value::Boolean(false))
    }
}
